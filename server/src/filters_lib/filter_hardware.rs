use axum::{extract::Path, response::IntoResponse, Json};
use serde_json::{json, Value};
use tokio::task;
use std::{sync::{Arc, Mutex}, collections::LinkedList};
use crate::HeaderMap;

use crate::main_lib::manage_vm::{get_vm_config};
use crate::main_lib::manage_pci::{get_pcis_info, add_pci_device, remove_pci_device};
use crate::main_lib::structure::{HostPci, HostGpu, RequestPciData, RequestGpuData, Ticket,
                                    generate_ticket, find_ticket, store_ticket, remove_ticket};

fn extract_addresses(raw: &Value, target: &str) -> Vec<String> {
    raw.as_array()
        .unwrap()
        .iter()
        .filter(|device| {
            let device_name = device["device_name"].as_str().unwrap();
            device_name.contains(target)
        })
        .map(|device| device["address"].as_str().unwrap().to_string())
        .collect()
}

pub async fn filter_get_vm_config(Path(vm_id): Path<String>) -> Json<Value> {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    let configs_json: Value = serde_json::from_str(&configs).unwrap();
    Json(configs_json)
}

pub async fn filter_pcis_info(filter: &str, arg1: &str) -> Json<Value> {
    println!("\nGetting the pcis info..");
    let devices = get_pcis_info(filter, arg1).await;
    Json(json!({ "devices": devices }))
}

pub async fn filter_add_pci(Path(vm_id): Path<String>, Json(payload): Json<RequestPciData>, 
                            ticket_list: Arc<Mutex<LinkedList<Ticket>>>) 
                            -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    println!("Generate the ticket id");
    let payload_str = serde_json::to_string(&payload).unwrap();
    let ticket_id = generate_ticket(vm_id, payload_str);
    let ticket_id_cloned = ticket_id.clone();
    let found_ticket = find_ticket(&ticket_id, &ticket_list);
    
    if found_ticket.is_none() {
        task::spawn(async move {
            let mut pcis_detail = Vec::new();
            for pci in payload.hostpcis {
                println!("\nTry passing through the device {}..", pci.address);
                // Detail example is "{"id":"_vfio3","bdf":"0000:00:06.0"}"
                let detail = add_pci_device(vm_id, &pci.address, 3);
                let pci_json: Value = serde_json::from_str(&detail).unwrap();
                pcis_detail.push(pci_json);
            }
            store_ticket(vm_id, &ticket_id, pcis_detail, &ticket_list);
        });
    }

    Json(json!({ 
        "ticket_id": ticket_id_cloned
    }))
   
    // Json(json!({ 
    //     "hostpcis": pcis_detail
    // }))
}

pub async fn filter_pt_status(headers: HeaderMap, ticket_list: Arc<Mutex<LinkedList<Ticket>>>) 
                                -> impl IntoResponse {
    println!("\nExtracting the ticket..");
    let ticket_id = headers.get("ticket").unwrap().to_str().unwrap();
    let found_ticket = find_ticket(&ticket_id, &ticket_list);

    let mut pcis_detail = Vec::new();
    if let Some(found_ticket) = found_ticket {
        println!("\nFound the ticket returning..");
        pcis_detail = found_ticket.pcis_detail;
        remove_ticket(&ticket_id, &ticket_list);
    }

    Json(json!({ 
        "hostpcis": pcis_detail
    }))
}

pub async fn filter_add_gpu(Path(vm_id): Path<String>, Json(payload): Json<RequestGpuData>) 
                            -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    println!("\nSearching for the gpu..");
    let raw = json!(get_pcis_info(&"class_code", "3").await);
    // let addresses = extract_addresses(&raw);

    let mut pcis_detail = Vec::new();
    for gpu in payload.hostgpus {
        let addresses = extract_addresses(&raw, &gpu.device_name);
        for i in 0..gpu.amount {
            // Skip if the resource is duplicated or busy
            let mut j = i;
            while j < addresses.len() as i32 {
                let detail = add_pci_device(vm_id, &addresses[j as usize], 3);
                if detail != "None" {
                    let pci_json: Value = serde_json::from_str(&detail).unwrap();
                    pcis_detail.push(pci_json);
                    break;
                }
                j += 1;
            }
        }
    }
   
    Json(json!({ 
       "hostpcis": pcis_detail
    }))
}

pub async fn filter_remove_pci(Path(vm_id): Path<String>, Json(payload): Json<RequestPciData>) -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    for pci in payload.hostpcis {
        println!("\nTry removing the passing through device {}..", pci.address);
        // Detail example is "{"id":"_vfio3","bdf":"0000:00:06.0"}"
        let _ = remove_pci_device(vm_id, &pci.address);
    }
   
    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    let configs_json: Value = serde_json::from_str(&configs).unwrap();

    Json(json!({ 
        "config": configs_json
    }))
}