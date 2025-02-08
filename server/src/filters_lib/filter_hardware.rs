use axum::{extract::Path, response::IntoResponse, Json};
use serde::{Deserialize};
use serde_json::{json, Value};

use crate::main_lib::manage_vm::{get_vm_config};
use crate::main_lib::manage_pci::{get_pcis_info, add_pci_device, remove_pci_device};

pub async fn filter_get_vm_config(Path(vm_id): Path<String>) -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    Json(json!({ "config_hashed": configs }))
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

    for pci in payload.hostpcis {
        println!("\nTry passing through the device {}..", pci.address);
        add_pci_device(vm_id, &pci.address);
    }

    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    Json(json!({ "config_hashed": configs }))
}