use std::{sync::{Arc, Mutex}, thread};
use axum::{extract::Path, http::{StatusCode}};

use crate::main_lib::structure::{VmStatus};
use crate::main_lib::manage_vm::{start_vm, force_terminate, delete_vm};

pub async fn filter_start_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                            Path(vm_id): Path<String>) -> StatusCode {
    
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    let config_path = format!("../vms-config/{}", vm_id);
    thread::spawn(move || {
        println!("\nRunning the VM..");
        let vm_status = start_vm(&vm_vec, vm_id, &config_path);
        if vm_status != 1 {
            println!("\nError: Cannot boot the VM.");
        }
    });

    StatusCode::ACCEPTED
}

pub async fn filter_stop_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                            Path(vm_id): Path<String>) -> StatusCode {
    
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    println!("\nForce terminating the vm..");
    force_terminate(&vm_vec, vm_id);
    StatusCode::ACCEPTED
}

pub async fn filter_restart_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                            Path(vm_id): Path<String>) -> StatusCode {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    println!("\nForce terminating the vm..");
    force_terminate(&vm_vec, vm_id);

    let config_path = format!("../vms-config/{}", vm_id);
    thread::spawn(move || {
        println!("\nRunning the VM..");
        let vm_status = start_vm(&vm_vec, vm_id, &config_path);
        if vm_status != 1 {
            println!("\nError: Cannot boot the VM.");
        }
    });

    StatusCode::ACCEPTED
}

pub async fn filter_delete_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                        Path(vm_id): Path<String>) -> StatusCode {

    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    println!("\nDeleting the vm..");
    delete_vm(&vm_vec, vm_id);
    StatusCode::ACCEPTED
}