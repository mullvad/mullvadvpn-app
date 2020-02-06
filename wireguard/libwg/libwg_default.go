// +build darwin linux
// +build !android

/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2020 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdlib.h>
import "C"
import (
	"bufio"
	"os"
	"strings"
	"unsafe"

	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun"
	
	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/logging"
	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/tunnelcontainer"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

//export wgTurnOn
func wgTurnOn(mtu int, cSettings *C.char, fd int, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)

	if cSettings == nil {
		logger.Error.Println("cSettings is null")
		return -1
	}
	settings := C.GoString(cSettings)

	file := os.NewFile(uintptr(fd), "")
	tunDevice, err := tun.CreateTUNFromFile(file, mtu)
	if err != nil {
		logger.Error.Println(err)
		if err.Error() == "bad file descriptor" {
			return -2
		}
		return -1
	}

	device := device.NewDevice(tunDevice, logger)

	err = device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if err != nil {
		logger.Error.Println(err)
		device.Close()
		return -2
	}

	device.Up()
	
	context := tunnelcontainer.Context {
		Device: device,
		Logger: logger,
	}
	
	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Error.Println(err)
		device.Close()
		return -1
	}
	
	return handle
}
