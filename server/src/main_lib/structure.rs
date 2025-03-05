use std::sync::{Arc, Mutex, MutexGuard};

pub const STATUS: [&str; 8] = ["Stopped", "Booting", "Running", "Unknown", 
                                "Stopping", "Paused", "Locked", "Migrating"]; 
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

pub fn mark_vm_stop(mut vm_vec: MutexGuard<'_, Vec<VmStatus>>, vm_id: usize) {
    vm_vec[vm_id].status = 0;
    vm_vec[vm_id].process_id = "".into();
    vm_vec[vm_id].lost_signal_count = 3;
    println!("vm_id: {} has no signal", vm_id);  
}