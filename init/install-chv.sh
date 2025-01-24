wget https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/v43.0/cloud-hypervisor
sudo mv cloud-hypervisor /bin/cloud-hypervisor
chmod +x /usr/bin/cloud-hypervisor 

mkdir ../vms-config
mkdir ../os
mkdir ../storage

wget https://github.com/cloud-hypervisor/rust-hypervisor-firmware/releases/download/0.4.2/hypervisor-fw -P ../os/
