use anyhow::bail;
use nix::unistd::pivot_root;
use std::{
    os::unix::prelude::PermissionsExt,
    path::Path,
    process::{exit, Command},
};

fn main() {
    // get arguments
    let args: Vec<String> = std::env::args().collect();

    // match if argument is parent or child and call parent and child functions
    match args[1].as_str() {
        "parent" => parent(),
        "child" => child(),
        _ => println!("Invalid argument"),
    }
}

fn provide_root_filesystem() -> anyhow::Result<()> {
    let new_root = "./rootfs";
    let rootfs = Path::new(new_root).join(".pivot_root/");

    // bind mount newroot to itself
    nix::mount::mount(
        Some(new_root),
        new_root,
        None::<&str>,
        nix::mount::MsFlags::MS_BIND,
        None::<&str>,
    )?;

    // create rootfs directory with 0700 permissions
    std::fs::create_dir_all(&rootfs)?;
    std::fs::set_permissions(&rootfs, std::fs::Permissions::from_mode(0o700))?;

    // call pivot_root
    pivot_root(new_root, &rootfs)?;

    // ensure current working directory is set to new
    std::env::set_current_dir("/")?;

    // unmount old rootfs
    nix::mount::umount2("/.pivot_root", nix::mount::MntFlags::MNT_DETACH)?;

    // remove old rootfs
    std::fs::remove_dir_all("/.pivot_root")?;

    Ok(())
}

fn mount_proc() -> anyhow::Result<()> {
    let new_root = "./rootfs";
    let target = Path::new(new_root).join("proc/");

    // make target dir and set permissions to 0755
    std::fs::create_dir_all(&target)?;
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755))?;

    // mount proc
    nix::mount::mount(
        Some("proc"),
        &target,
        Some("proc"),
        nix::mount::MsFlags::empty(),
        None::<&str>,
    )?;

    Ok(())
}

fn parent() {
    // vector with namespaces to unshare (i.e., mount, uts, ipc, pid, net, and user)
    let namespaces = vec![
        unshare::Namespace::Mount,
        unshare::Namespace::Uts,
        unshare::Namespace::Ipc,
        unshare::Namespace::Pid,
        unshare::Namespace::Net,
        unshare::Namespace::User,
    ];

    let uid_map = unshare::UidMap {
        inside_uid: 0,
        outside_uid: 1000,
        count: 1,
    };

    let gid_map = unshare::GidMap {
        inside_gid: 0,
        outside_gid: 1000,
        count: 1,
    };

    let mut executor = unshare::Command::new("/proc/self/exe");

    let executor = executor.arg("child").arg("/bin/sh");

    let executor = executor.unshare(&namespaces);

    let executor = executor.set_id_maps(vec![uid_map], vec![gid_map]);

    // This program depends on uidmap being installed on the system - to install it on Ubuntu, run: `sudo apt install uidmap`
    let executor = executor.set_id_map_commands("/usr/bin/newuidmap", "/usr/bin/newgidmap");

    let mut child = executor.spawn().unwrap_or_else(|e| {
        eprintln!("Failed to spawn process: {}", e);
        exit(1);
    });

    // wait for child to exit
    let _ = child.wait().unwrap_or_else(|e| {
        eprintln!("Failed to wait for child: {}", e);
        exit(1);
    });
}

fn child() {
    let mut executor = unshare::Command::new("/bin/sh");

    mount_proc().unwrap_or_else(|e| {
        eprintln!("Failed to mount proc: {}", e);
        exit(1);
    });

    // get pid
    let pid = std::process::id();

    // set hostname to container-<pid>
    let _ = Command::new("hostname")
        .arg(format!("container-{}", pid))
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run process: {}", e);
            exit(1);
        });

    provide_root_filesystem().unwrap_or_else(|e| {
        eprintln!("Failed to provide root filesystem: {}", e);
        exit(1);
    });

    let mut child = executor.spawn().unwrap_or_else(|e| {
        eprintln!("Failed to spawn process: {}", e);
        exit(1);
    });

    // wait for child to exit
    let _ = child.wait().unwrap_or_else(|e| {
        eprintln!("Failed to wait for child: {}", e);
        exit(1);
    });
}
