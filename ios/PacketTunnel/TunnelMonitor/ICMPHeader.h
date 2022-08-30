//
//  ICMPHeader.h
//  MullvadVPN
//
//  Created by pronebird on 24/08/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

#ifndef ICMPHEADER_H
#define ICMPHEADER_H

struct ICMPHeader {
    uint8_t type;
    uint8_t code;
    uint16_t checksum;
    uint16_t identifier;
    uint16_t sequenceNumber;
    // data...
} __attribute__((packed));
typedef struct ICMPHeader ICMPHeader;

__Check_Compile_Time(sizeof(ICMPHeader) == 8);

#endif /* ICMPHEADER_H */
