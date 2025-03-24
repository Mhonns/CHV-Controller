sudo nano /etc/default/grub
# I added iommu=pt and amd_iommu=on

sudo update-grub

# # Option 1: Does not work so far
# sudo nano /etc/initramfs-tools/scripts/init-top/vfio.sh

# # I added
# # #!/bin/sh

# # PREREQ=""

# # prereqs()
# # {
# #    echo "$PREREQ"
# # }

# # case $1 in
# # prereqs)
# #    prereqs
# #    exit 0
# #    ;;
# # esac

# # for dev in 0000:23:00.0
# # do 
# #  echo "vfio-pci" > /sys/bus/pci/devices/$dev/driver_override 
# #  echo "$dev" > /sys/bus/pci/drivers/vfio-pci/bind 
# # done

# # exit 0

# sudo chmod +x /etc/initramfs-tools/scripts/init-top/vfio.sh

# sudo nano /etc/initramfs-tools/modules

# # I added
# # options kvm ignore_msrs=1

# sudo update-initramfs -u -k all

# # Option 2
# GRUB_CMDLINE_LINUX_DEFAULT="amd_iommu=on iommu=pt kvm.ignore_msrs=1 vfio-pci.ids=