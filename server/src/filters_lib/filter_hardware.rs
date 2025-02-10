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

pub async fn filter_pcis_info() -> Json<Value> {
    println!("\nGetting the pcis info..");
    let devices = get_pcis_info().await;
    Json(json!({ "devices": devices }))
}

#[derive(Deserialize)]
pub struct HostPci {
    address: String,
    class_code: u32,
    device_id: u32,
    revision: u32,
    subclass_code: u32,
    vendor_id: u32,
}

#[derive(Deserialize)]
pub struct RequestData {
    hostpcis: Vec<HostPci>,
}

pub async fn filter_add_pci(Path(vm_id): Path<String>, Json(payload): Json<RequestData>) -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    let mut pcis_detail = Vec::new();
    for pci in payload.hostpcis {
        println!("\nTry passing through the device {}..", pci.address);
        // Detail example is "{"id":"_vfio3","bdf":"0000:00:06.0"}"
        let detail = add_pci_device(vm_id, &pci.address);
        let pci_json: Value = serde_json::from_str(&detail).unwrap();
        pcis_detail.push(pci_json);
    }
   
    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    let configs_json: Value = serde_json::from_str(&configs).unwrap();
    Json(json!({ 
        "config": configs_json,
        "hostpcis": pcis_detail
    }))
}

pub async fn filter_remove_pci(Path(vm_id): Path<String>, Json(payload): Json<RequestData>) -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    let mut pcis_detail = Vec::new();
    for pci in payload.hostpcis {
        println!("\nTry removing the passing through device {}..", pci.address);
        // Detail example is "{"id":"_vfio3","bdf":"0000:00:06.0"}"
        let detail = remove_pci_device(vm_id, &pci.address);
        let pci_json: Value = serde_json::from_str(&detail).unwrap();
        pcis_detail.push(pci_json);
    }
   
    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    let configs_json: Value = serde_json::from_str(&configs).unwrap();
    Json(json!({ 
        "config": configs_json
    }))
}