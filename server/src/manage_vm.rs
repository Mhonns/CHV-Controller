use crate::lib::lib::{
    MAXVM,
    VmStatus,
    init_vm_vec,
    find_free_slot,
    modify_slot,
    i16_to_usize,
};

use std::{
    sync::{Arc, Mutex},
    process::Command,
    ffi::OsStr,
    time::Duration,
    net::{TcpStream, SocketAddr},
    fs,
    thread,
};

use sysinfo::{System};

pub fn start_vm(config_path: &str) -> i32 {
    match Command::new("sh").arg("-c")
        .arg(format!("sudo sh {}/vm-config.sh", config_path))
        .output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
                    return -1;
                }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                return -1;
            }
        }
    return 1;
}

// TODO mark the vm slot as stop
pub fn stop_vm(vm_id: i16) {
    let pid_str = get_vm_proc_id(vm_id);
    if pid_str != "None" {
        let kill_status = Command::new("sudo")
            .arg("kill")
            .arg("-9")
            .arg(pid_str.clone())
            .status()
            .expect("Failed to execute kill command");

        if kill_status.success() {
            println!("The process {} was terminated", pid_str);
        } else {
            println!("There is a problem while removing and end the process");
        }
    } else {
        println!("There is no process for vm id: {}", vm_id);
    }

    let remove_api_status = Command::new("sudo")
            .arg("rm")
            .arg("-rf")
            .arg(format!("/tmp/cloud-hypervisor{}.sock", vm_id))
            .status()
            .expect("Failed to execute remove command");
    
    if remove_api_status.success() {
        println!("The api socket was removed");
    }
}

pub fn get_vm_proc_id(vm_id: i16) -> String {
    let mut s = System::new_all();
    s.refresh_all();

    let search_command = format!("/tmp/cloud-hypervisor{}.sock", vm_id); 
    let s = System::new_all();
    for process in s.processes_by_name(OsStr::new("cloud-h")) {
        println!("Process Info:\n\
                PID: {}\n\
                Name: {}\n\
                Memory: {} KB\n\
                CPU Usage: {}%\n",
                process.pid(),
                process.name().to_string_lossy(),
                process.memory(),
                process.cpu_usage());
        if process.cmd().iter().any(|cmd| cmd.to_string_lossy().contains(&search_command)) {
            return process.pid().to_string();
        }
    }
    return "None".to_string();
}

// TODO mark the vm slot as deleted
pub fn delete_vm(vm_id: i16) {
    stop_vm(vm_id);
    let config_path = format!("../vms-config/{}", vm_id);
    let storage_path = format!("../storage/cloudinit{}.img", vm_id);
    let _ = fs::remove_dir_all(config_path);
    let _ = fs::remove_file(storage_path);
}

pub fn resize_storage(config_path: &str, image: &str, storage: &str) {
    println!("{}", format!("qemu-img resize {}/{}.raw +{}", config_path, image, storage));
    match Command::new("sh").arg("-c")
        .arg(format!("qemu-img resize {}/{}.raw +{}", 
                        config_path, image, storage))
        .output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => eprintln!("Failed to execute command: {}", e),
        }
}


fn check_port(ip: &str, port: u16) -> bool {
    // Construct the address to connect to
    let addr = format!("{}:{}", ip, port);

    // Set a timeout to avoid hanging indefinitely
    let timeout = Duration::new(2, 0); // Timeout of 2 seconds

    // Attempt to connect to the IP address and port
    match TcpStream::connect_timeout(&addr.parse::<SocketAddr>().unwrap(), timeout) {
        Ok(_) => {
            println!("Port {} on {} is open.", port, ip);
            true
        }
        Err(_) => {
            println!("Port {} on {} is closed.", port, ip);
            false
        }
    }
}

pub fn monitor_vm(vm_vec: &Arc<Mutex<Vec<VmStatus>>>, vm_id: &i16) {
    // Initialize the structure and wait
    let proc_id = get_vm_proc_id(*vm_id);
    let usize_vm_id = i16_to_usize(*vm_id);
    modify_slot(vm_vec, usize_vm_id, &proc_id, 1);
    let ip = format!("192.168.{}.2", *vm_id);

    // Booting fault torelence
    let mut boot_attempt = 3;
    let mut success = false; 
    while !success { 
        // Boot failed
        if boot_attempt < 0 {
            delete_vm(*vm_id);
            break;
        }
        boot_attempt -= 1;

        // Ping attempt
        let mut ping_attempt = 10;
        thread::sleep_ms(40000);
        while true { 
            thread::sleep_ms(4000);
            if ping_attempt > 0 {
                if check_port(&ip, 22) {
                    success = true;
                    break;
                } else {
                    ping_attempt -= 1;
                }
            } else {
                break;
            }
        } 
    }
}