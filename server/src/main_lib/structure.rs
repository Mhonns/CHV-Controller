use std::{sync::{Arc, Mutex, MutexGuard}, collections::LinkedList};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::{Digest, Sha1};

pub const STATUS: [&str; 8] = ["Stopped", "Booting", "Running", "Unknown", 
                                "Stopping", "Paused", "Locked", "Migrating"]; 
pub const MAXVM: usize = 253;

// VM structure
pub struct VmStatus {
    pub process_id: Box<str>,
    pub status: i32,
    pub lost_signal_count: usize
}

// Host resource structure
// pub struct HostResource {
//     pub cpu_cores: usize,
//     pub memory: f64,
//     pub storage: f64,
//     // devices avail
// }

// Device hardware and ticket structure
#[derive(Deserialize, Serialize)]
pub struct HostPci {
    pub address: String,
}

#[derive(Deserialize, Serialize)]
pub struct HostGpu {
    pub device_name: String,
    pub amount: i32,
}

#[derive(Deserialize, Serialize)]
pub struct RequestPciData {
    pub hostpcis: Vec<HostPci>,
}

#[derive(Deserialize, Serialize)]
pub struct RequestGpuData {
    pub hostgpus: Vec<HostGpu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,      
    pub vm_id: i16,      
    pub pcis_detail: Vec<Value>, 
}

// Ticket function
pub fn generate_ticket(vm_id: i16, payload_str: String) -> String {
    // Generate SHA-1 hash
    let mut hasher = Sha1::new();
    hasher.update(format!("{}{}", vm_id, payload_str));
    let ticket_id = format!("{:x}", hasher.finalize());

    // Log the added ticket
    ticket_id
}

pub fn store_ticket(vm_id: i16, ticket_id: &str, pcis_detail: Vec<Value>,
                    ticket_list: &Arc<Mutex<LinkedList<Ticket>>>) {
    // Create a new ticket
    let ticket = Ticket {
        id: ticket_id.to_string(),
        vm_id,
        pcis_detail: pcis_detail,
    };

    {
        let mut list = ticket_list.lock().unwrap();
        list.push_back(ticket.clone());
    }
}

pub fn find_ticket(ticket_id: &str, ticket_list: &Arc<Mutex<LinkedList<Ticket>>>) -> Option<Ticket> {
    let list = ticket_list.lock().unwrap();

    for ticket in list.iter() {
        if ticket.id == ticket_id {
            return Some(ticket.clone());
        }
    }

    None
}

pub fn remove_ticket(ticket_id: &str, ticket_list: &Arc<Mutex<LinkedList<Ticket>>>) -> bool {
    let mut list = ticket_list.lock().unwrap(); // Lock the list for thread safety
    let original_len = list.len(); // Store original length before modification

    // Rebuild the list, skipping the ticket with matching ID
    *list = list.iter()
        .filter(|ticket| ticket.id != ticket_id) // Keep only tickets that don’t match
        .cloned()
        .collect();

    // Return true if a ticket was removed, false if not found
    original_len != list.len()
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