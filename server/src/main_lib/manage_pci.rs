use pci_info::{PciInfo};
use serde_json::{json, Value};

pub async fn get_pcis_info() -> Vec<Value> {
    let info = match PciInfo::enumerate_pci() {
        Ok(devices) => devices,
        Err(_) => return vec![json!({"error": "Failed to enumerate PCI devices"})],
    };

    let mut devices = Vec::new();

    for r in info {
        if let Ok(device) = r {
            let location = device
                .location()
                .map(|loc| format!("{:02x}:{:02x}.{:x}", loc.bus(), loc.device(), loc.function()))
                .unwrap_or_else(|_| "Unknown".to_string());
            
            let device_json = json!({
                "address": location,
                "vendor_id": device.vendor_id(),
                "device_id": device.device_id(),
                "revision": device.revision().unwrap_or(0),
                "class_code": device.device_class_code().unwrap_or(0),
                "subclass_code": device.device_subclass_code().unwrap_or(0)
            });

            devices.push(device_json);
        } else {
            eprintln!("Error reading device information");
        }
    }

    devices
}