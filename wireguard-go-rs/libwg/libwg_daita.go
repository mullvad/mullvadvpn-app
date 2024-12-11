//go:build daita
// +build daita

/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2024 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdio.h>
// #include <stdlib.h>
// #include <stdint.h>
import "C"

import (
	"unsafe"

	"golang.zx2c4.com/wireguard/device"
)

//export wgActivateDaita
func wgActivateDaita(tunnelHandle C.int32_t, peerPubkey *C.uint8_t, machines *C.char, maxPaddingFrac C.double, maxBlockingFrac C.double, eventsCapacity C.uint32_t, actionsCapacity C.uint32_t) C.int32_t {

	tunnel, err := tunnels.Get(int32(tunnelHandle))
	if err != nil {
		return ERROR_UNKNOWN_TUNNEL
	}

	var publicKey device.NoisePublicKey
	copy(publicKey[:], C.GoBytes(unsafe.Pointer(peerPubkey), device.NoisePublicKeySize))

	var peer *device.Peer
	if tunnel.EntryDevice != nil {
		// TODO: Document me
		peer = tunnel.EntryDevice.LookupPeer(publicKey)
	} else {
		// TODO: Document me
		peer = tunnel.Device.LookupPeer(publicKey)
	}

	if peer == nil {
		return ERROR_UNKNOWN_PEER
	}

	if !peer.EnableDaita(goStringFixed((*C.char)(machines)), uint(eventsCapacity), uint(actionsCapacity), float64(maxPaddingFrac), float64(maxBlockingFrac)) {
		return ERROR_ENABLE_DAITA
	}

	return OK
}
