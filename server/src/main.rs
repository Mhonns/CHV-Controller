use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
    time::Duration,
    sync::atomic::{AtomicU8, Ordering},
    thread,
    fs,
};

// VM funcitons
mod init_vm;
use init_vm::{
    get_cloud_image,
    write_cloud_config,
    create_cloud_init_files,
    write_vm_config,
    run_cloud_init,
    start_vm,
    resize_storage,
};

fn main() {
    let listener = TcpListener::bind("154.215.14.240:2546").unwrap();
    let thread_counter = Arc::new(AtomicU8::new(0));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let thread_id = thread_counter.fetch_add(1, Ordering::SeqCst) % 255;

        thread::spawn(move || {
            handle_connection(stream, thread_id);
        });
    }   
}

fn extract_url(url: &str) -> Vec<String>  {
    // Extract only parameter
    let params = url
        .split('?')
        .nth(1)         // Get part after '?'
        .unwrap_or("")  // Handle case where '?' isn't found
        .split(" ")     // Split by space
        .next()         // Get first part
        .unwrap_or(""); // Handle case where space isn't found
    
    let results = params.split('&')
        .filter_map(|param| {
            param.split_once('=')
                .map(|(_, value)| value.to_string())
        })
        .collect();
    
    return results
}

fn handle_connection(stream: TcpStream, vm_id: u8) {
    let buf_reader = BufReader::new(&stream);
    let mut lines = buf_reader.lines();
    let request_line = lines.next().unwrap().unwrap();
    // request_line = "GET /vm/create HTTP/1.1"

    if request_line.starts_with("GET /vm/create") {
        // Default values
        let mut image = "focal-server-cloudimg-amd64".to_string();
        let mut cpu = 4;
        let mut ram = 16;
        let mut storage = "1G".to_string();
        let mut username = String::new();
        let mut password = String::new();

        println!("\nExtract the url..");
        let values = extract_url(&request_line);
        match (values.get(0), values.get(1), values.get(2), values.get(3)) {
            (Some(image_str), Some(cpu_str), Some(ram_str), Some(storage_str)) => {
                // For image, just assign the string
                image = image_str.to_string();
                
                // Parse numeric values
                if let Ok(cpu_val) = cpu_str.parse() {
                    cpu = cpu_val;
                }
                if let Ok(ram_val) = ram_str.parse() {
                    ram = ram_val;
                }
                // For storage, just assign the string
                storage = storage_str.to_string();

                println!("Image: {}", image);
                println!("CPU: {}", cpu);
                println!("RAM: {}", ram);
                println!("Storage: {}", storage);
            },
            _ => println!("Not enough values provided"),
        }

        // Read headers
        while let Some(Ok(line)) = lines.next() {
            if line.is_empty() {
                break;
            }
            
            if let Some((key, value)) = line.split_once(": ") {
                if key.to_string() == "username" {
                    username = value.to_string();
                } else if key.to_string() == "password" {
                    password = value.to_string();
                }
            }
        }
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
        
        println!("\n Running the VM..");
        let cloud_status = run_cloud_init(&config_path);
        let vm_status = start_vm(&config_path);
        if vm_status != 1 && cloud_status != 1 {
            println!("\nError can not boot the vm..");
        }
    } else if request_line == "GET /vms/status HTTP/1.1" {
        // println!("The current VM ID {}", vm_id);
        for i in 1..20 {
            println!("hi number {i} from the main thread!");
            thread::sleep(Duration::from_millis(1000));
        }
    }
}