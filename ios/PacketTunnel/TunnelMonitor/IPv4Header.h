//
//  IPv4Header.h
//  MullvadVPN
//
//  Created by pronebird on 24/08/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

#ifndef IPV4HEADER_H
#define IPV4HEADER_H

#include <stdint.h>
#include <AssertMacros.h>

struct IPv4Header {
    uint8_t versionAndHeaderLength;
    uint8_t differentiatedServices;
    uint16_t totalLength;
    uint16_t identification;
    uint16_t flagsAndFragmentOffset;
    uint8_t timeToLive;
    uint8_t protocol;
    uint16_t headerChecksum;
    uint8_t sourceAddress[4];
    uint8_t destinationAddress[4];
    // options...
    // data...
} __attribute__((packed));
typedef struct IPv4Header IPv4Header;

__Check_Compile_Time(sizeof(IPv4Header) == 20);

#endif /* IPV4HEADER_H */
