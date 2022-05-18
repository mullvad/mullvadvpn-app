/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdlib.h>
import "C"

import (
	"bufio"
	"bytes"
	"runtime"
	"strings"
	"unsafe"

	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/tunnelcontainer"
)

const (
	ERROR_GENERAL_FAILURE      = -1
	ERROR_INTERMITTENT_FAILURE = -2
)

var tunnels tunnelcontainer.Container

func init() {
	tunnels = tunnelcontainer.New()
}

//export wgTurnOff
func wgTurnOff(tunnelHandle int32) {
	{
		tunnel, err := tunnels.Remove(tunnelHandle)
		if err != nil {
			return
		}
		tunnel.Device.Close()
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
func wgSetConfig(tunnelHandle int32, cSettings *C.char) int32 {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	if cSettings == nil {
		tunnel.Logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}
	settings := C.GoString(cSettings)

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
