sudo iptables -t nat -A POSTROUTING -o eno12409np1 -j MASQUERADE
sudo iptables -A FORWARD -i eno12409np1 -o ens4 -m state --state RELATED,ESTABLISHED -j ACCEPT
sudo iptables -A FORWARD -i ens4 -o eno12409np1 -j ACCEPT
