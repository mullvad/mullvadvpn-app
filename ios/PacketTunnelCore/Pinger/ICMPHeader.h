// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

#ifndef ICMPHEADER_H
#define ICMPHEADER_H

#include <AssertMacros.h>

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
