echo 'deb http://download.opensuse.org/repositories/home:/cloud-hypervisor/xUbuntu_22.04/ /' | sudo tee /etc/apt/sources.list.d/home:cloud-hypervisor.list
curl -fsSL https://download.opensuse.org/repositories/home:cloud-hypervisor/xUbuntu_22.04/Release.key | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/home_cloud-hypervisor.gpg > /dev/null
sudo apt update
sudo apt install cloud-hypervisor
