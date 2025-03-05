use crate::main_lib::structure::{
    MAXVM,
    VmStatus,
};

use std::{
    sync::{Arc, Mutex},
    process::Command,
    ffi::OsStr,
    time::Duration,
    fs,
    thread,
};

use crate::main_lib::structure::{mark_vm_stop};
use sysinfo::System;
// use sha1::{Sha1, Digest};

const INTERVAL: u64 = 15000;

pub fn start_vm(vm_vec: &Arc<Mutex<Vec<VmStatus>>>, vm_id: i16, config_path: &str) -> i32 {
    {
        let mut vm_vec = vm_vec.lock().unwrap();
        vm_vec[vm_id as usize].status = 1;
    }
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
                {
                    let mut vm_vec = vm_vec.lock().unwrap();
                    vm_vec[vm_id as usize].status = -1;
                }
                return -1;
            }
        }
    return 1;
}

pub fn shutdown_vm(vm_vec: &Arc<Mutex<Vec<VmStatus>>>, vm_id: i16) {
    {
        let mut vm_vec = vm_vec.lock().unwrap();
        vm_vec[vm_id as usize].status = 4;
    }
    let api_socket = format!("/tmp/cloud-hypervisor{}.sock", vm_id);
    let output = Command::new("sudo")
        .arg("ch-remote")
        .arg("--api-socket")
        .arg(api_socket)
        .arg("shutdown")
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let _ = String::from_utf8(output.stdout).unwrap();
            } else {
                eprintln!(
                    "Command failed with exit code: {:?}\nError: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
        }
    }
}

pub fn force_terminate(vm_vec: &Arc<Mutex<Vec<VmStatus>>>, vm_id: i16) {
    let vm_vec = vm_vec.lock().unwrap();
    let pid_str: String = (vm_vec[vm_id as usize].process_id).clone().into();
    if pid_str != "None" {
        let kill_status = Command::new("sudo")
            .arg("kill")
            .arg("-9")
            .arg(pid_str.clone())
            .status()
            .expect("Failed to execute kill command");

        if kill_status.success() {
            println!("The process {} was terminated", pid_str);
            mark_vm_stop(vm_vec, vm_id as usize);
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

pub fn get_vm_config(vm_id: i16) -> String {
    let api_socket = format!("/tmp/cloud-hypervisor{}.sock", vm_id);
    let output = Command::new("sudo")
        .arg("ch-remote")
        .arg("--api-socket")
        .arg(api_socket)
        .arg("info")
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8(output.stdout).unwrap();
                return output_str
            } else {
                eprintln!(
                    "Command failed with exit code: {:?}\nError: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                );
                return "Error: Can not get the config for this vm".to_string()
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            return "Error: Can not get the config for this vm".to_string()
        }
    }
}

pub fn get_vm_proc_id(vm_id: i16) -> String {
    let mut s = System::new_all();
    s.refresh_all();

    let search_command = format!("/tmp/cloud-hypervisor{}.sock", vm_id); 
    let s = System::new_all();
    for process in s.processes_by_name(OsStr::new("cloud-h")) {
        // println!("Process Info:\n\
        //         PID: {}\n\
        //         Name: {}\n\
        //         Memory: {} KB\n\
        //         CPU Usage: {}%\n",
        //         process.pid(),
        //         process.name().to_string_lossy(),
        //         process.memory(),
        //         process.cpu_usage());
        if process.cmd().iter().any(|cmd| cmd.to_string_lossy().contains(&search_command)) {
            return process.pid().to_string();
        }
    }
    return "None".to_string();
}

pub fn delete_vm(vm_vec: &Arc<Mutex<Vec<VmStatus>>>, vm_id: i16) {
    force_terminate(&vm_vec, vm_id);
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

pub async fn monitor_vms(vm_vec: &Arc<Mutex<Vec<VmStatus>>>) {
    loop {
        // Sleeping
        let wait_delay = Duration::from_millis(INTERVAL);
        thread::sleep(wait_delay);

        // Find the all vm_id
        for vm_id in 0..MAXVM {

            // VM: Running case -> mark no signal
            let mut vm_vec = vm_vec.lock().unwrap();
            if vm_vec[vm_id].status > 0 {
                let addr = format!("192.168.{}.2", vm_id).parse().unwrap();
                let data = [1,2,3];
                let timeout = Duration::from_secs(1);
                let options = ping_rs::PingOptions { ttl: 128, dont_fragment: true };
                let result = ping_rs::send_ping(&addr, timeout, &data, Some(&options));
                match result {
                    Ok(_) => {
                        vm_vec[vm_id].status = 2;
                        if vm_vec[vm_id].process_id == "".into() {
                            vm_vec[vm_id].process_id = get_vm_proc_id(vm_id.try_into().unwrap()).into();
                        }
                    },
                    Err(_) => {
                        if vm_vec[vm_id].status != 1 {
                            vm_vec[vm_id].status = 3;
                        }
                    }
                }
            }
            // VM: No signal case  -> mark stop
            if vm_vec[vm_id].status == 3 || vm_vec[vm_id].status == 4 {
                if vm_vec[vm_id].lost_signal_count > 0 {
                    vm_vec[vm_id].lost_signal_count -= 1;
                } else {
                    mark_vm_stop(vm_vec, vm_id);
                    println!("vm_id: {} has no signal", vm_id);  
                }
            }
        }
    }
}
