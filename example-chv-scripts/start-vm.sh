cloud-hypervisor \
    --api-socket /tmp/cloud-hypervisor.sock \
    --kernel ../os/hypervisor-fw \
    --disk path=../os/focal-server-cloudimg-amd64.raw path=/tmp/ubuntu-cloudinit.img \
    --cpus boot=4 \
    --memory size=16G \
    --net "tap=vmtap0,mac=ae:00:22:d0:d9:6f,ip=192.168.249.1,mask=255.255.255.0"
