//go:build windows
// +build windows

/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2025 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdlib.h>
// #include <stdint.h>
// #include <string.h>
import "C"

import (
	"bufio"
	"strings"
	"unsafe"

	"golang.org/x/sys/windows"

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
func wgTurnOn(cIfaceName *C.char, cIfaceNameOut *C.char, cIfaceNameOutSize C.size_t, cLuidOut *C.uint64_t, mtu C.uint16_t, cSettings *C.char, logSink LogSink, logContext LogContext) C.int32_t {
	logger := logging.NewLogger(logSink, logging.LogContext(logContext))

	if cIfaceName == nil {
		logger.Errorf("cIfaceName is null\n")
		return ERROR_GENERAL_FAILURE
	}

	if cSettings == nil {
		logger.Errorf("cSettings is null\n")
		return ERROR_GENERAL_FAILURE
	}

	settings := C.GoString(cSettings)
	ifaceName := C.GoString(cIfaceName)

	// {AFE43773-E1F8-4EBB-8536-576AB86AFE9A}
	networkId := windows.GUID{
		Data1: 0xafe43773,
		Data2: 0xe1f8,
		Data3: 0x4ebb,
		Data4: [8]byte{0x85, 0x36, 0x57, 0x6a, 0xb8, 0x6a, 0xfe, 0x9a},
	}

	tun.WintunTunnelType = "Mullvad"

	wintun, err := tun.CreateTUNWithRequestedGUID(ifaceName, &networkId, int(mtu))
	if err != nil {
		logger.Errorf("Failed to create tunnel\n")
		logger.Errorf("%s\n", err)
		return ERROR_INTERMITTENT_FAILURE
	}

	nativeTun := wintun.(*tun.NativeTun)

	actualInterfaceName, err := nativeTun.Name()
	if err != nil {
		nativeTun.Close()
		logger.Errorf("Failed to determine name of wintun adapter\n")
		return ERROR_GENERAL_FAILURE
	}
	if actualInterfaceName != ifaceName {
		// WireGuard picked a different name for the adapter than the one we expected.
		// This indicates there is already an adapter with the name we intended to use.
		logger.Verbosef("Failed to create adapter with specific name\n")
	}

	device := device.NewDevice(wintun, conn.NewDefaultBind(), logger)

	setError := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		logger.Errorf("Failed to set device configuration\n")
		logger.Errorf("%s\n", setError)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

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

	if cIfaceNameOut != nil {
		if int(cIfaceNameOutSize) <= len(actualInterfaceName) {
			logger.Errorf("Interface name buffer too small\n")
			device.Close()
			return ERROR_GENERAL_FAILURE
		}
		cName := C.CString(actualInterfaceName)
		C.strcpy(cIfaceNameOut, cName)
		C.free(unsafe.Pointer(cName))
	}
	if cLuidOut != nil {
		*cLuidOut = C.uint64_t(nativeTun.LUID())
	}

	return C.int32_t(handle)
}

//export wgUpdateBind
func wgUpdateBind() {
	tunnels.ForEach(func(tunnel tunnelcontainer.Context) {
		tunnel.Device.BindUpdate()
	})
}
