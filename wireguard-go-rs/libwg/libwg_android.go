/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdint.h>
import "C"

import (
	"bufio"
	"strings"
	"unsafe"

	"golang.org/x/sys/unix"

	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun"

	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/logging"
	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/tunnelcontainer"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = C.uint64_t

//export wgTurnOn
func wgTurnOn(cSettings *C.char, fd int, logSink LogSink, logContext LogContext) C.int32_t {
	logger := logging.NewLogger(logSink, logging.LogContext(logContext))

	if cSettings == nil {
		logger.Errorf("cSettings is null\n")
		return ERROR_INVALID_ARGUMENT
	}

	settings := goStringFixed(cSettings)

	tunDevice, _, err := tun.CreateUnmonitoredTUNFromFD(fd)
	if err != nil {
		logger.Errorf("%s\n", err)
		unix.Close(fd)
		if err.Error() == "bad file descriptor" {
			return ERROR_INTERMITTENT_FAILURE
		}
		return ERROR_GENERAL_FAILURE
	}

	device := device.NewDevice(tunDevice, conn.NewStdNetBind(), logger)

	setErr := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setErr != nil {
		logger.Errorf("%s\n", setErr)
		device.Close()
		return ERROR_INTERMITTENT_FAILURE
	}

	device.DisableSomeRoamingForBrokenMobileSemantics()
	device.Up()

	context := tunnelcontainer.Context{
		Device: device,
		Logger: logger,
	}

	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Errorf("%s\n", err)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	return C.int32_t(handle)
}

//export wgGetSocketV4
func wgGetSocketV4(tunnelHandle int32) C.int32_t {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return ERROR_UNKNOWN_TUNNEL
	}
	peek := tunnel.Device.Bind().(conn.PeekLookAtSocketFd)
	fd, err := peek.PeekLookAtSocketFd4()
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	return C.int32_t(fd)
}

//export wgGetSocketV6
func wgGetSocketV6(tunnelHandle int32) C.int32_t {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return ERROR_UNKNOWN_TUNNEL
	}
	peek := tunnel.Device.Bind().(conn.PeekLookAtSocketFd)
	fd, err := peek.PeekLookAtSocketFd6()
	if err != nil {
		return ERROR_GENERAL_FAILURE
	}
	return C.int32_t(fd)
}
