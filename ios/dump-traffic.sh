function examineTraffic {
    local host="$1"
    ssh root@192.168.1.1 \
        tcpdump -i any -U -s0 -w - "host $host" | \
        wireshark -k -i -
}

examineTraffic $@
