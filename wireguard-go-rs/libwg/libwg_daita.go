//go:build daita
// +build daita

package main

// #include <stdio.h>
// #include <stdlib.h>
// #include "libwg.h"
import "C"

import (
	"unsafe"

	"golang.zx2c4.com/wireguard/device"
)

//export wgActivateDaita
func wgActivateDaita(tunnelHandle int32, noisePublic *C.uint8_t, machines *C.char, eventsCapacity uint32, actionsCapacity uint32) C.bool {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return false
	}

	tunnel.Logger.Verbosef("Initializing libmaybenot")
	var publicKey device.NoisePublicKey
	copy(publicKey[:], C.GoBytes(unsafe.Pointer(noisePublic), device.NoisePublicKeySize))
	peer := tunnel.Device.LookupPeer(publicKey)

	if peer == nil {
		return false
	}

	return (C.bool)(peer.EnableDaita(C.GoString((*C.char)(machines)), uint(eventsCapacity), uint(actionsCapacity)))
}
