sudo iptables -t nat -A POSTROUTING -o eno12399np0 -j MASQUERADE
sudo iptables -A FORWARD -i eno12399np0 -o vmtap0 -m state --state RELATED,ESTABLISHED -j ACCEPT
sudo iptables -A FORWARD -i vmtap0 -o eno12399np0 -j ACCEPT
