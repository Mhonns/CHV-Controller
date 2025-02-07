use std::sync::{Arc, Mutex};

pub const STATUS: [&str; 5] = ["Stopped", "Booting", "Running", "No Signal", "Stopping"]; 
pub const MAXVM: usize = 253;

pub struct VmStatus {
    pub process_id: Box<str>,
    pub status: i32,
    pub lost_signal_count: usize
}

pub fn init_vm_vec(vm_vec: &Arc<Mutex<Vec<VmStatus>>>) {
    let mut vm_vec = vm_vec.lock().unwrap();

    for _ in 0..MAXVM {
        vm_vec.push(VmStatus {
            process_id: String::from("").into_boxed_str(),
            status: -1,
            lost_signal_count: 2,
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
