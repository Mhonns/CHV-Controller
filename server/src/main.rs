use std::{
    io::{prelude::*, BufReader, Lines},
    net::{TcpListener, TcpStream},
    time::Duration,
    sync::{Arc, Mutex},
    thread,
    fs,
};

mod manage_pci;
use manage_pci::{
    get_pci_all_info,
};

// Structure
mod lib;
use crate::lib::lib::{
    MAXVM,
    VmStatus,
    STATUS,
    init_vm_vec,
    find_free_slot,
    modify_slot,
    i16_to_usize,
};=

// VM init funcitons
mod init_vm;
use init_vm::{
    get_cloud_image,
    write_cloud_config,
    create_cloud_init_files,
    write_vm_config,
    run_cloud_init,
};

// VM manage funcitons
mod manage_vm;
use manage_vm:: {
    start_vm,
    resize_storage,
    stop_vm,
    delete_vm,
    monitor_vm,
};

fn main() {
    let listener = TcpListener::bind("154.215.14.240:2546").unwrap();
    let vm_vec: Arc<Mutex<Vec<VmStatus>>> = Arc::new(Mutex::new(Vec::with_capacity(MAXVM)));
    let vm_vec_clone = Arc::clone(&vm_vec);
    init_vm_vec(&vm_vec_clone);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let vm_vec_clone = Arc::clone(&vm_vec);
        thread::spawn(move || {
            thread_filtering(stream, &vm_vec_clone);
        });
    }   
}

fn get_vm_id_from_header(mut lines: Lines<BufReader<&TcpStream>>) -> i16 {
    while let Some(Ok(line)) = lines.next() {
        if line.is_empty() {
            break;
        }
        
        if let Some((key, value)) = line.split_once(": ") {
            if key.to_string() == "vm_id" {
                return value.parse().unwrap();
            }
        }
    }
    return -1;
}

fn handle_creating(vm_id: i16, image: &str, cpu: i32, ram: i32, storage: &str,
                    username: &str, password: &str) {
    println!("Image: {}", image);
    println!("CPU: {}", cpu);
    println!("RAM: {}", ram);
    println!("Storage: {}", storage);
    println!("Username: {}", username);
    println!("Password: {}", password);

    println!("\nCreating config directory..");
    let config_path = format!("../vms-config/{}", vm_id);
    let _ = fs::create_dir_all(config_path.clone());

    println!("\nDownloading the cloud image..");
    get_cloud_image(&config_path, &image);

    println!("\nWriting the VM starting config..");
    let ip_gw = format!("192.168.{}.1", vm_id);
    let ip = format!("192.168.{}.2", vm_id);
    let _ = write_vm_config(vm_id, &config_path, &image, cpu, ram);
    let _ = write_cloud_config(vm_id, &config_path);
    let _ = create_cloud_init_files(&config_path, &username, &password, &ip, &ip_gw);
    resize_storage(&config_path, &image, &storage);
    
    println!("\nRunning the VM..");
    let cloud_status = run_cloud_init(&config_path);
    let vm_status = start_vm(&config_path);
    if vm_status != 1 && cloud_status != 1 {
        println!("\nError can not boot the vm..");
    }
}

fn thread_filtering(mut stream: TcpStream, vm_vec: &Arc<Mutex<Vec<VmStatus>>>) {
    let buf_reader = BufReader::new(&stream);
    let mut lines = buf_reader.lines();
    let request_line = lines.next().unwrap().unwrap();

    // Create the vm
    if request_line.starts_with("POST /api/v1/vm") {
        // Default values
        let mut image = "focal-server-cloudimg-amd64".to_string();
        let mut cpu = 4;
        let mut ram = 16;
        let mut storage = "1G".to_string();
        let mut username = String::new();
        let mut password = String::new();

        // Read headers
        println!("\nRead the http header..");
        while let Some(Ok(line)) = lines.next() {
            if line.is_empty() {
                break;
            }
            
            if let Some((key, value)) = line.split_once(": ") {
                let key_str = key.to_string();
                let val_str = value.to_string();
                if key_str == "image" {
                    image = val_str;
                } else if key_str == "cpu" {
                    cpu = value.parse().unwrap();
                } else if key_str == "ram" {
                    ram = value.parse().unwrap();
                } else if key_str == "storage" {
                    storage = val_str;
                } else if key_str == "username" {
                    username = val_str;
                } else if key_str == "password" {
                    password = val_str;
                }
            }
        }

        let vm_id = find_free_slot(vm_vec);
        if vm_id != -1 {
            {
                let status_line = "HTTP/1.1 200 OK";
                let contents = format!("Your vm ID: {}", vm_id);
                let length = contents.len();
                let response = format!(
                    "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
                );
                stream.write_all(response.as_bytes()).unwrap();
            }   

            let vm_vec_clone = Arc::clone(&vm_vec);
            thread::spawn(move || {
                monitor_vm(&vm_vec_clone, &vm_id);
            });
            handle_creating(vm_id, &image, cpu, ram, 
                            &storage, &username, &password);
        } else {
            println!("Error: The number of vm reached the maximum.");
        }

    // Get the vm status
    } else if request_line.starts_with("GET /api/v1/vm") {
        println!("\nRead the http header..");
        let vm_id = get_vm_id_from_header(lines);

        // {
        //     let mut vm_vec = vm_vec.lock().unwrap();
        
        //     let status = vm_vec[i16_to_usize(vm_id)].status;
        //     let status_line = "HTTP/1.1 200 OK";
        //     let contents = format!("Your vm status {}", STATUS[status]);
        //     let length = contents.len();
        //     let response = format!(
        //         "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
        //     );
        //     stream.write_all(response.as_bytes()).unwrap();
        // }
        
    // Update the vm
    } else if request_line.starts_with("PUT /api/v1/vm") {
        println!("\nRead the http header..");
        let vm_id = get_vm_id_from_header(lines);

        println!("\nRunning the VM {}..", vm_id);
        let config_path = format!("../vms-config/{}", vm_id);
        let vm_status = start_vm(&config_path);
    
    // Delete the vm
    } else if request_line.starts_with("DELETE /api/v1/vm") {
        println!("\nRead the http header..");
        let vm_id = get_vm_id_from_header(lines);
        
        println!("\nDeleting the VM..");
        delete_vm(vm_id);
    
    } else if request_line.starts_with("GET /thread_test") {
        for i in 1..20 {
            println!("hi number {i} from the main thread!");
            thread::sleep(Duration::from_millis(1000));
        }
    }
}