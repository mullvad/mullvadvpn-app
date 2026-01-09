// SPDX-License-Identifier: GPL-2.0-or-later

#include <linux/bpf.h>

// "mole", magic fwmark number used to exclude packets from the tunnel
const __u32 FWMARK = 0x6d6f6c65;
