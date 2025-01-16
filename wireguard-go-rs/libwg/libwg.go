/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2025 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdio.h>
// #include <stdlib.h>
// #include <stdint.h>
import "C"

import (
	"bufio"
	"bytes"
	"runtime"
	"strings"
	"unsafe"

	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/tunnelcontainer"
	"golang.zx2c4.com/wireguard/device"
)

// FFI integer result codes
// NOTE: Must be kept in sync with the Error enum in wireguard-go-rs
const (
	OK = C.int32_t(-iota)

	// Something went wrong.
	ERROR_GENERAL_FAILURE

	// Something went wrong, but trying again might help.
	ERROR_INTERMITTENT_FAILURE

	// A bad argument was provided to libwg.
	ERROR_INVALID_ARGUMENT

	// The provided tunnel handle did not refer to an existing tunnel.
	ERROR_UNKNOWN_TUNNEL

	// The provided public key did not refer to an existing peer.
	ERROR_UNKNOWN_PEER

	// Something went wrong when enabling DAITA.
	ERROR_ENABLE_DAITA
)

var tunnels tunnelcontainer.Container

func init() {
	tunnels = tunnelcontainer.New()
}

type EventContext struct {
	tunnelHandle int32
	peer         device.NoisePublicKey
}

//export wgTurnOff
func wgTurnOff(tunnelHandle int32) {
	{
		tunnel, err := tunnels.Remove(tunnelHandle)
		if err != nil {
			return
		}
		tunnel.Device.Close()
		if tunnel.EntryDevice != nil {
			tunnel.EntryDevice.Close()
		}
	}
	// Calling twice convinces the GC to release NOW.
	runtime.GC()
	runtime.GC()
}

//export wgGetConfig
func wgGetConfig(tunnelHandle int32) *C.char {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return nil
	}
	settings := new(bytes.Buffer)
	writer := bufio.NewWriter(settings)
	if err := tunnel.Device.IpcGetOperation(writer); err != nil {
		tunnel.Logger.Errorf("Failed to get config for tunnel: %s\n", err)
		return nil
	}
	writer.Flush()
	return C.CString(settings.String())
}

//export wgSetConfig
func wgSetConfig(tunnelHandle int32, cSettings *C.char) C.int32_t {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return ERROR_UNKNOWN_TUNNEL
	}
	if cSettings == nil {
		tunnel.Logger.Errorf("cSettings is null\n")
		return ERROR_INVALID_ARGUMENT
	}
	settings := goStringFixed(cSettings)

	setError := tunnel.Device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		tunnel.Logger.Errorf("Failed to set device configuration\n")
		tunnel.Logger.Errorf("%s\n", setError)
		return ERROR_GENERAL_FAILURE
	}
	return 0
}

//export wgFreePtr
func wgFreePtr(ptr unsafe.Pointer) {
	C.free(ptr)
}

func main() {}

//Instead of using the normal version of C.GoString we need to use the version that takes
//a length argument (C.GoStringN). That is because the normal C.GoString reads a whole
//page of memory to determine the length of the string. This causes a crash if
//mte is turned on. So instead we determine the length of the c string by reading
//each character until we reach the end of the string.
func goStringFixed(cString *C.char) string {
	ptr := unsafe.Pointer(cString)
	i := 0
	for {
		byte := (*C.char)(unsafe.Pointer(uintptr(ptr) + uintptr(i)))

		if *byte == 0 {
			break
		}

		i += 1
	}

	return C.GoStringN(cString, C.int(i))
}
