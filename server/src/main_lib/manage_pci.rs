use pci_info::{PciInfo};
use serde_json::{json, Value};
use std::process::Command;
use pci_ids::{Device, FromId, Vendor};

pub async fn get_pcis_info(filter: &str, arg1: &str) -> Vec<Value> {
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
            
            let vendor_id = device.vendor_id();
            let device_id = device.device_id();
            let class_code = device.device_class_code().unwrap_or(0);
            let subclass_code = device.device_subclass_code().unwrap_or(0);

            if filter == "class_code" && arg1 != class_code.to_string() { continue };
            
            // Look up vendor and device names using pci-ids
            let vendor_name = match Vendor::from_id(vendor_id) {
                Some(v) => v.name().to_string(),
                None => format!("vendor ({:04x})", vendor_id),
            };
            
            let device_name = match Device::from_vid_pid(vendor_id, device_id) {
                Some(d) => d.name().to_string(),
                None => format!("Unknown device ({:04x})", device_id),
            };
            
            let device_json = json!({
                "address": location,
                "vendor_id": vendor_id,
                "device_id": device_id,
                "vendor_name": vendor_name,
                "device_name": format!("{} {}", vendor_name, device_name),
                "revision": device.revision().unwrap_or(0),
                "class_code": class_code,
                "subclass_code": subclass_code
            });

            devices.push(device_json);
        } else {
            eprintln!("Error reading device information");
        }
    }

    devices
}

pub fn add_pci_device(vm_id: i16, device_id: &str, retries: i16) -> String {
    let mut result: String = "None".to_string();
    let mut retries = retries;
    while retries > 0 && result == "None" {
        let api_socket = format!("/tmp/cloud-hypervisor{}.sock", vm_id);
        let output = Command::new("sudo")
            .arg("ch-remote")
            .arg("--api-socket")
            .arg(api_socket)
            .arg("add-device")
            .arg(format!("path=/sys/bus/pci/devices/0000:{}/", device_id))
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    result = String::from_utf8(output.stdout).unwrap();
                    println!("Set the virtual machine configuration successfully.");
                } else {
                    eprintln!(
                        "Command failed with exit code: {:?}\nError: {}",
                        output.status.code(),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    retries -= 1;
                    result = "None".to_string();
                }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                retries -= 1;
                result = "None".to_string();
            }
        }
    }

    return result
}

pub fn remove_pci_device(vm_id: i16, device_id: &str) -> String {
    let api_socket = format!("/tmp/cloud-hypervisor{}.sock", vm_id);
    let output = Command::new("sudo")
        .arg("ch-remote")
        .arg("--api-socket")
        .arg(api_socket)
        .arg("remove-device")
        .arg(format!("{}", device_id))
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8(output.stdout).unwrap();
                println!("Set the virtual machine configuration successfully.");
                return output_str
            } else {
                eprintln!(
                    "Command failed with exit code: {:?}\nError: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                );
                return "None".to_string();
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            return "None".to_string();
        }
    }
}
