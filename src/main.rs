use std::process::{exit, Command};

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

fn parent() {
    // parent says hello
    println!("Hello from parent");

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
    // child says hello
    println!("Hello from child");

    let mut executor = unshare::Command::new("/bin/sh");

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
