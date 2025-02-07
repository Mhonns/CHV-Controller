use std::{sync::{Arc, Mutex}, fs, thread};
use axum::{Router, extract::Path, routing::{post, get, put, delete}, http::{HeaderMap,  StatusCode}, 
            response::IntoResponse, Json};
use serde::Serialize;
use serde_json::{json, Value};

mod main_lib;
use main_lib::structure::{init_vm_vec, find_free_slot, STATUS, VmStatus, MAXVM};
use main_lib::init_vm::{get_cloud_image, write_cloud_config, create_cloud_init_files, write_vm_config, run_cloud_init};
use main_lib::manage_vm::{start_vm, resize_storage, force_terminate, delete_vm, get_vm_config, monitor_vms};
use main_lib::manage_pci::get_pcis_info;


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

async fn filter_start_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                            Path(vm_id): Path<String>) -> StatusCode {
    
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    let config_path = format!("../vms-config/{}", vm_id);
    thread::spawn(move || {
        println!("\nRunning the VM..");
        let vm_status = start_vm(&vm_vec, vm_id, &config_path);
        if vm_status != 1 {
            println!("\nError: Cannot boot the VM.");
        }
    });

    StatusCode::ACCEPTED
}

async fn filter_stop_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                            Path(vm_id): Path<String>) -> StatusCode {
    
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    println!("\nForce terminating the vm..");
    force_terminate(&vm_vec, vm_id);
    StatusCode::ACCEPTED
}

async fn filter_restart_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                            Path(vm_id): Path<String>) -> StatusCode {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    println!("\nForce terminating the vm..");
    force_terminate(&vm_vec, vm_id);

    let config_path = format!("../vms-config/{}", vm_id);
    thread::spawn(move || {
        println!("\nRunning the VM..");
        let vm_status = start_vm(&vm_vec, vm_id, &config_path);
        if vm_status != 1 {
            println!("\nError: Cannot boot the VM.");
        }
    });

    StatusCode::ACCEPTED
}

async fn filter_delete_vm(vm_vec: Arc<Mutex<Vec<VmStatus>>>, 
                        Path(vm_id): Path<String>) -> StatusCode {

    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::METHOD_NOT_ALLOWED,
    };

    println!("\nDeleting the vm..");
    delete_vm(&vm_vec, vm_id);
    StatusCode::ACCEPTED
}

async fn filter_pcis_info() -> Json<Value> {
    println!("\nGetting the pcis info..");
    let devices = get_pcis_info().await;
    Json(json!({ "devices": devices }))
}

async fn filter_get_vm_config(Path(vm_id): Path<String>) -> impl IntoResponse {
    println!("\nValidating the vm id..");
    let vm_id: i16 = match vm_id.parse() {
        Ok(id) => id,
        Err(_) => return Json(json!({"Error": vm_id})),
    };

    println!("\nGetting the vm config..");
    let configs = get_vm_config(vm_id);
    Json(json!({ "configs": configs }))
}


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Init vm data structure
    let vm_vec: Arc<Mutex<Vec<VmStatus>>> = Arc::new(Mutex::new(Vec::with_capacity(MAXVM)));
    init_vm_vec(&vm_vec);

    // Routing configure
    let node_name = "0";
    let vmm_str = format!("/api/v1/nodes/{}/vmm", node_name);
    let binding = vmm_str.clone() + "/{vm_id}/config";
    let vm_config_str = binding.as_str();
    let pci_str = format!("/api/v1/nodes/{}/vmm/hardware/pci", node_name);
    let app = Router::new()
        // Create and get status
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
        // Individuals vm
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
            (vmm_str.clone() + "/{vm_id}/restart").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_restart_vm(vm_vec, path)
            }),
        )
        .route(
            (vmm_str.clone() + "/{vm_id}/delete").as_str(),
            post({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_delete_vm(vm_vec, path)
            }),
        )
        // Virtual machine configuration
        .route(
            vm_config_str,
            get({
                move |path| filter_get_vm_config(path)
            }),
        )
        .route(
            vm_config_str,
            put({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_restart_vm(vm_vec, path)
            }),
        )
        .route(
            vm_config_str,
            delete({
                let vm_vec = Arc::clone(&vm_vec);
                move |path| filter_restart_vm(vm_vec, path)
            }),
        )
        // Hardware
        .route(
            pci_str.as_str(),
            get({
                filter_pcis_info().await
            }),
        );
        

    // Spawn monitoring as a task
    let _ = tokio::spawn({
        let vm_vec_clone = Arc::clone(&vm_vec);
        async move {
            monitor_vms(&vm_vec_clone).await;
        }
    });

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2546").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}