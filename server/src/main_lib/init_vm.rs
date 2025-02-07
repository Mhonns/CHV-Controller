use std::{
    path::Path,
    process::Command,
    io::Write,
    fs,
    fs::OpenOptions,
};

pub fn get_cloud_image(config_path: &str, image: &str) {
    let og_file_path = &format!("../os/{}.img", image);
    let file_exist = Path::new(og_file_path).exists();
    if !file_exist {
        match Command::new("sh").arg("-c")
        .arg(format!("wget https://cloud-images.ubuntu.com/focal/current/{}.img -P ../os/", image))
        .output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => eprintln!("Failed to execute command: {}", e),
        }
    }
    else {
        println!("[Skipped] Cloud image found locally");
    }

    let image_name = image.split(".").nth(0).unwrap_or("");
    match Command::new("sh").arg("-c")
        .arg(format!("qemu-img convert -p -f qcow2 -O raw {} {}/{}.raw", 
                        og_file_path, config_path, image_name))
        .output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => eprintln!("Failed to execute command: {}", e),
        }
}

pub fn write_cloud_config(vm_id: i16, config_path: &str) -> std::io::Result<()> {
    let file_path = format!("{}/cloud-config.sh", config_path);
    let content = format!(r#"#!/usr/bin/env bash
set -x

rm -f ../storage/cloudinit{}.img
mkdosfs -n CIDATA -C ../storage/cloudinit{}.img 8156
mcopy -oi ../storage/cloudinit{}.img -s {}/user-data ::
mcopy -oi ../storage/cloudinit{}.img -s {}/meta-data ::
mcopy -oi ../storage/cloudinit{}.img -s {}/network-config ::"#,
    vm_id, vm_id, vm_id, config_path, vm_id, config_path, vm_id, config_path);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;

    file.write_all(content.as_bytes())?;

    println!("File written successfully!");
    Ok(())
}

pub fn create_cloud_init_files(config_path: &str, username: &str, password: &str, 
                            ip: &str, ip_gw: &str) -> std::io::Result<()> {
    // Create user-data content
    let user_data = format!(
        "#cloud-config
users:
  - name: {}
    passwd: {}
    sudo: ALL=(ALL) NOPASSWD:ALL
    lock_passwd: False
    inactive: False
    shell: /bin/bash

ssh_pwauth: True
",
        username, password
    );

    // Create meta-data content
    let meta_data = "instance-id: cloud\n\
                     local-hostname: cloud\n";

    // Create network-config content
    let network_config = format!(
        "version: 2
ethernets:
  ens4:
    match:
       macaddress: ae:00:22:d0:d9:6f
    addresses: [{}/24]
    gateway4: {}
",
        ip, ip_gw
    );

    // Write to files
    let mut file = fs::File::create(format!("{}/user-data", config_path))?;
    file.write_all(user_data.as_bytes())?;

    let mut file = fs::File::create(format!("{}/meta-data", config_path))?;
    file.write_all(meta_data.as_bytes())?;

    let mut file = fs::File::create(format!("{}/network-config", config_path))?;
    file.write_all(network_config.as_bytes())?;

    Ok(())
}

pub fn write_vm_config(vm_id: i16, config_path: &str, image: &str, cpu: i32, ram: i32) 
                        -> std::io::Result<()> {
    let file_path = format!("{}/vm-config.sh", config_path);
    let ip = format!("ip=192.168.{}.1,mask=255.255.255.0", vm_id);
    let content = format!(r#"cloud-hypervisor \
    --api-socket /tmp/cloud-hypervisor{}.sock \
    --kernel ../os/hypervisor-fw \
    --disk path={}/{}.raw path=../storage/cloudinit{}.img \
    --cpus boot={} \
    --memory size={}G \
    --net "tap=vmtap{},mac=ae:00:22:d0:d9:6f,{}""#, 
    vm_id, config_path, image, vm_id, cpu, ram, vm_id, ip);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;

    // Write content to the file
    file.write_all(content.as_bytes())?;

    println!("File written successfully!");
    Ok(())
}

pub fn run_cloud_init(config_path: &str) -> i32 {
    match Command::new("sh").arg("-c")
        .arg(format!("sudo sh {}/cloud-config.sh", config_path))
        .output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
                    return -1;
                }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                return -1;
            }
        }
    return 1;
}