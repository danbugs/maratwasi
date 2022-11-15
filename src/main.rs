use std::process::{exit, Command};

fn main() {
    // unshare the mount, uts, ipc, pid, net, and user namespaces
    let mut unshare = Command::new("unshare")
        .arg("--fork")
        .arg("--mount")
        .arg("--uts")
        .arg("--ipc")
        .arg("--pid")
        .arg("--net")
        .arg("--user")
        .arg("/bin/sh")
        .spawn()
        .expect("failed to execute unshare");
    
    let pid = unshare.id();
    println!("unshare pid: {}", pid);
    
    // // set newuidmap to 0 1000 1 for pid
    // let mut cmd = Command::new("newuidmap")
    //     .arg(pid.to_string())
    //     .arg("0")
    //     .arg("1000")
    //     .arg("1")
    //     .status()
    //     .expect("failed to execute newuidmap");

    // // set newgidmap to 0 1000 1 for pid
    // let mut cmd = Command::new("newgidmap")
    //     .arg(pid.to_string())
    //     .arg("0")
    //     .arg("1000")
    //     .arg("1")
    //     .status()
    //     .expect("failed to execute newgidmap");

    // // set hostname to "container-<pid>"
    // let hostname = format!("container-{}", pid);
    // Command::new("hostname")
    //     .arg(hostname)
    //     .status()
    //     .expect("failed to execute hostname");
    
    // wait for the child process to exit
    let status = unshare.wait().expect("failed to wait on child");
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}
