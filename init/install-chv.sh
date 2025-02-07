wget https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/v44.0/cloud-hypervisor
wget https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/v44.0/ch-remote
sudo mv cloud-hypervisor /bin/cloud-hypervisor
sudo mv ch-remote /bin/ch-remote
chmod +x /usr/bin/cloud-hypervisor 
chmod +x /usr/bin/ch-remote

mkdir ../vms-config
mkdir ../os
mkdir ../storage

wget https://github.com/cloud-hypervisor/rust-hypervisor-firmware/releases/download/0.4.2/hypervisor-fw -P ../os/
