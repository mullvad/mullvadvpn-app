//go:build daita
// +build daita

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
func wgActivateDaita(tunnelHandle C.int32_t, noisePublic *C.uint8_t, machines *C.char, eventsCapacity C.uint32_t, actionsCapacity C.uint32_t) C.int32_t {
	tunnel, err := tunnels.Get(int32(tunnelHandle))
	if err != nil {
		return ERROR_UNKNOWN_TUNNEL
	}

	tunnel.Logger.Verbosef("Initializing libmaybenot")
	var publicKey device.NoisePublicKey
	copy(publicKey[:], C.GoBytes(unsafe.Pointer(noisePublic), device.NoisePublicKeySize))
	peer := tunnel.Device.LookupPeer(publicKey)

	if peer == nil {
		return ERROR_UNKNOWN_PEER
	}

	if !peer.EnableDaita(C.GoString((*C.char)(machines)), uint(eventsCapacity), uint(actionsCapacity)) {
		return ERROR_ENABLE_DAITA
	}

	return OK
}
