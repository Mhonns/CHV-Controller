use std::{sync::{Arc, Mutex}, fs, thread, collections::LinkedList};
use axum::{Router, extract::Path, routing::{post, get, put, delete}, http::{HeaderMap}, 
            response::IntoResponse, Json};
use serde::{Serialize};
use serde_json::{json};

// Main libraries
mod main_lib;
use main_lib::structure::{STATUS, MAXVM, VmStatus, Ticket, init_vm_vec, find_free_slot};
use main_lib::init_vm::{get_cloud_image, write_cloud_config, create_cloud_init_files, 
                        write_vm_config, run_cloud_init};
use main_lib::manage_vm::{start_vm, resize_storage, monitor_vms};

// Preprocessing libraries
mod filters_lib;
use filters_lib::filter_vm_manage::{filter_start_vm, filter_stop_vm, filter_reboot_vm, 
                                    filter_delete_vm, filter_shutdown_vm};
use filters_lib::filter_hardware::{filter_get_vm_config, filter_pcis_info, filter_add_pci, 
                                    filter_add_gpu, filter_remove_pci, filter_pt_status};

#[derive(Serialize)]
struct VmInfo {
    vm_id: usize,
    status: Box<str>,
}

async fn create_vm(headers: HeaderMap, vm_vec: Arc<Mutex<Vec<VmStatus>>>) -> Json<serde_json::Value> {
    // Extract variable
    let image = headers.get("image").unwrap().to_str().unwrap();
    let cpu = headers.get("cpu").unwrap().to_str().unwrap().parse::<i32>().unwrap();
    let ram = headers.get("ram").unwrap().to_str().unwrap().parse::<i32>().unwrap();
    let storage = headers.get("storage").unwrap().to_str().unwrap();
    let username = headers.get("username").unwrap().to_str().unwrap();
    let password = headers.get("password").unwrap().to_str().unwrap();

    let vm_id = find_free_slot(&vm_vec);
    if vm_id < 0 {
        return Json(json!({"Error": vm_id}));
    }

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

    thread::spawn(move || {
        println!("\nRunning the VM..");
        let cloud_status = run_cloud_init(&config_path);
        let vm_status = start_vm(&vm_vec, vm_id, &config_path);
        if vm_status != 1 && cloud_status != 1 {
            println!("\nError: Cannot boot the VM.");
        } 

        {
            let vm_vec = &mut vm_vec.lock().unwrap();
            vm_vec[vm_id as usize].status = 0;
        }
        
        return
    });
    
    Json(json!({
        "vm_id": vm_id,
    }))
}

async fn get_vms_info(vm_vec: Arc<Mutex<Vec<VmStatus>>>) -> impl IntoResponse{
    let vm_vec = vm_vec.lock().unwrap();
    let mut vm_info_list = Vec::new();

    for vm_id in 0..MAXVM {
        if vm_vec[vm_id].status > -1 {
            let vm_status: &str = STATUS[vm_vec[vm_id].status as usize];
            vm_info_list.push(VmInfo {
                vm_id,
                status: vm_status.into(),
            });
        }
    }

    Json(vm_info_list)
}

async fn get_vm_status(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                        Path(vm_id): Path<String>) -> Json<serde_json::Value> {
    let vm_vec = vm_vec.lock().unwrap();

    println!("\nValidating the vm id..");
    let vm_id: usize = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return 
            Json(json!({
                "vm_id": "Error",
                "status": "Method is not allowed",
            })),
    };
    
    println!("\nGetting the VM status..");
    let mut vm_status = "Not Found";
    if vm_vec[vm_id].status >= 0 {
        vm_status = STATUS[vm_vec[vm_id].status as usize];
    }   

    Json(json!({
        "vm_id": vm_id,
        "status": vm_status,
    }))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Init data structure
    let vm_vec: Arc<Mutex<Vec<VmStatus>>> = Arc::new(Mutex::new(Vec::with_capacity(MAXVM)));
    init_vm_vec(&vm_vec);
    let ticket_list: Arc<Mutex<LinkedList<Ticket>>> = Arc::new(Mutex::new(LinkedList::new()));

    // Spawn monitoring as a task
    let _ = tokio::spawn({
        let vm_vec_clone = Arc::clone(&vm_vec);
        async move {
            monitor_vms(&vm_vec_clone).await;
        }
    });

    // Routing configure
    let node_name = "0";
    let vmm_str = format!("/api/v1/nodes/{}/vmm", node_name);
    let binding = vmm_str.clone() + "/{vm_id}/config";
    let vm_config_str = binding.as_str();
    let pci_str = format!("/api/v1/nodes/{}/vmm/hardware/pci", node_name);
    let app = Router::new()
        // Create and get status VMM
        .route(
            vmm_str.as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |headers| create_vm(headers, vm_vec)
            }),
        )
        .route(
            vmm_str.as_str(),
            get({
                let vm_vec = Arc::clone(&vm_vec);
                move || async move { get_vms_info(vm_vec).await }
            }),
        )
        // Individuals VM
        .route(
            (vmm_str.clone() + "/{vm_id}/status").as_str(),
            get({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| get_vm_status(vm_vec, path)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/start").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_start_vm(vm_vec, path)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/stop").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_stop_vm(vm_vec, path)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/reboot").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_reboot_vm(vm_vec, path)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/shutdown").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_shutdown_vm(vm_vec, path)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/delete").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_delete_vm(vm_vec, path)
            }),
        )
        // Hardware
        .route(
            vm_config_str,
            get(move |path| filter_get_vm_config(path)),
        )
        .route(
            vm_config_str,
            put({
                let ticket_list = Arc::clone(&ticket_list);
                move |path, json_data| filter_add_pci(path, json_data, ticket_list)
            }),
        )
        .route(
            vm_config_str,
            delete({
                move |path, json_data| filter_remove_pci(path, json_data)
            }),
        )
        .route(
            pci_str.as_str(),
            get( filter_pcis_info(&"", &"").await ),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/pt_status").as_str(),
            get({
                let ticket_list = Arc::clone(&ticket_list);
                move |headers| filter_pt_status(headers, ticket_list)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/gpus").as_str(),
            put(move |path, json_data| filter_add_gpu(path, json_data)),
        );

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2546").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}