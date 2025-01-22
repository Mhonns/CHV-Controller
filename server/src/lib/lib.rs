use std::sync::{Arc, Mutex};

pub const STATUS: [&str; 4] = ["Stopped", "Creating", "Running", "Stopping"]; 
pub const MAXVM: usize = 253;

pub struct VmStatus {
    pub process_id: Box<str>,
    pub status: i32,
}

pub fn init_vm_vec(vm_vec: &Arc<Mutex<Vec<VmStatus>>>) {
    let mut vm_vec = vm_vec.lock().unwrap();

    for _ in 0..MAXVM {
        vm_vec.push(VmStatus {
            process_id: String::from("").into_boxed_str(),
            status: -1,
        });
    }
}

pub fn find_free_slot(vm_vec: &Arc<Mutex<Vec<VmStatus>>>) -> i16 {
    let mut vm_vec = vm_vec.lock().unwrap();

    for i in 0..MAXVM {
        if vm_vec[i].status == -1 {
            vm_vec[i].status = 0; 
            return i.try_into().unwrap();
        }
    }
    return -1;
}

pub fn modify_slot(vm_vec: &Arc<Mutex<Vec<VmStatus>>>, vm_id: usize,
                    process_id: &str, status: i32) {
    let mut vm_vec = vm_vec.lock().unwrap();
    vm_vec[vm_id].process_id = process_id.into(); 
    vm_vec[vm_id].status = status;
}

pub fn i16_to_usize(value: i16) -> usize {
    if value < 0 {
        0
    } else {
        value as usize
    }
}