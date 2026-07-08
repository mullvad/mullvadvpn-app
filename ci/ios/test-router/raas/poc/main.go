package main

import (
    "encoding/binary"
    "log"

    "github.com/songgao/water"
)

func main() {
    ifce, err := water.New(water.Config{
        DeviceType: water.TUN,
    })
    if err != nil {
        log.Fatal(err)
    }
    log.Printf("Created: %s", ifce.Name())

    buf := make([]byte, 1500)
    for {
        n, err := ifce.Read(buf)
        if err != nil {
            log.Print(err)
            continue
        }
        pkt := buf[:n]

        if len(pkt) < 20 || pkt[9] != 1 {
            continue
        }

        ihl := int(pkt[0]&0x0f) * 4

        if len(pkt) < ihl+4 || pkt[ihl] != 8 {
            continue
        }

        pkt[ihl] = 0
        pkt[ihl+2] = 0
        pkt[ihl+3] = 0

        icmpLen := n - ihl
        csum := onesComplementSum(pkt[ihl : ihl+icmpLen])
        binary.BigEndian.PutUint16(pkt[ihl+2:], csum)

        src := make([]byte, 4)
        copy(src, pkt[12:16])
        copy(pkt[12:16], pkt[16:20])
        copy(pkt[16:20], src)

        ifce.Write(pkt)
        log.Printf("Replied to ping")
    }
}

func onesComplementSum(data []byte) uint16 {
    var sum uint32
    for i := 0; i < len(data)-1; i += 2 {
        sum += uint32(binary.BigEndian.Uint16(data[i:]))
    }
    if len(data)%2 == 1 {
        sum += uint32(data[len(data)-1]) << 8
    }
    sum = (sum >> 16) + (sum & 0xffff)
    sum += (sum >> 16)
    return ^uint16(sum)
}

