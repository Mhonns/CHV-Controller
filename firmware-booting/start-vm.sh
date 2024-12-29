cloud-hypervisor \
    --api-socket /tmp/cloud-hypervisor.sock \
    --kernel ./hypervisor-fw \
    --disk path=focal-server-cloudimg-amd64.raw path=/tmp/ubuntu-cloudinit.img \
    --cpus boot=4 \
    --memory size=16G \
    --net "tap=vmtap0,mac=,ip=,mask="
