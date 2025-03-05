use axum::{extract::Path, response::IntoResponse, Json};
use serde::{Deserialize};
use serde_json::{json, Value};

use crate::main_lib::manage_vm::{get_vm_config};
use crate::main_lib::manage_pci::{get_pcis_info, add_pci_device, remove_pci_device};

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

#[derive(Deserialize)]
pub struct HostPci {
    address: String,
}

#[derive(Deserialize)]
pub struct HostGpu {
    device_name: String,
    amount: i32,
}

#[derive(Deserialize)]
pub struct RequestPciData {
    hostpcis: Vec<HostPci>,
}

#[derive(Deserialize)]
pub struct RequestGpuData {
    hostgpus: Vec<HostGpu>,
}

pub async fn filter_add_pci(Path(vm_id): Path<String>, Json(payload): Json<RequestPciData>) 
                            -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    let mut pcis_detail = Vec::new();
    for pci in payload.hostpcis {
        println!("\nTry passing through the device {}..", pci.address);
        // Detail example is "{"id":"_vfio3","bdf":"0000:00:06.0"}"
        let detail = add_pci_device(vm_id, &pci.address, 3);
        let pci_json: Value = serde_json::from_str(&detail).unwrap();
        pcis_detail.push(pci_json);
    }
   
    Json(json!({ 
        "hostpcis": pcis_detail
    }))
}

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