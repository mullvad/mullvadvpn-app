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
	"fmt"
	"strings"
	"unsafe"

	"golang.org/x/sys/windows"

	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun"
	"golang.zx2c4.com/wireguard/tun/wintun"
	"golang.zx2c4.com/wireguard/windows/tunnel/winipcfg"

	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/interfacewatcher"
	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/logging"
	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/tunnelcontainer"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

var MullvadPool *wintun.Pool

func init() {
	var err error
	MullvadPool, err = wintun.MakePool("Mullvad")
	if err != nil {
		panic(fmt.Errorf("Failed to make pool: %w", err))
	}
}

func createInterfaceWatcherEvents(waitOnIpv6 bool, tunLuid uint64) []interfacewatcher.Event {
	if waitOnIpv6 {
		return []interfacewatcher.Event{
			{
				Luid:   winipcfg.LUID(tunLuid),
				Family: windows.AF_INET,
			},
			interfacewatcher.Event {
				Luid:   winipcfg.LUID(tunLuid),
				Family: windows.AF_INET6,
			},
		}
	} else {
		return []interfacewatcher.Event{
			{
				Luid:   winipcfg.LUID(tunLuid),
				Family: windows.AF_INET,
			},
		}
	}
}

//export wgTurnOn
func wgTurnOn(cIfaceName *C.char, mtu int, waitOnIpv6 bool, cSettings *C.char, cIfaceNameOut **C.char, logSink LogSink, logContext LogContext) int32 {
	logger := logging.NewLogger(logSink, logContext)
	if cIfaceNameOut != nil {
		*cIfaceNameOut = nil
	}

	if cIfaceName == nil {
		logger.Error.Println("cIfaceName is null")
		return ERROR_GENERAL_FAILURE
	}

	if cSettings == nil {
		logger.Error.Println("cSettings is null")
		return ERROR_GENERAL_FAILURE
	}

	settings := C.GoString(cSettings)
	ifaceName := C.GoString(cIfaceName)

	// {AFE43773-E1F8-4EBB-8536-576AB86AFE9A}
	networkId := windows.GUID{0xafe43773, 0xe1f8, 0x4ebb, [8]byte{0x85, 0x36, 0x57, 0x6a, 0xb8, 0x6a, 0xfe, 0x9a}}

	watcher, err := interfacewatcher.NewWatcher()
	if err != nil {
		logger.Error.Println(err)
		return ERROR_GENERAL_FAILURE
	}
	defer watcher.Destroy()

	if tun.WintunPool != MullvadPool {
		tun.WintunPool = MullvadPool
	}

	wintun, err := tun.CreateTUNWithRequestedGUID(ifaceName, &networkId, mtu)
	if err != nil {
		logger.Error.Println("Failed to create tunnel")
		logger.Error.Println(err)
		return ERROR_GENERAL_FAILURE
	}

	nativeTun := wintun.(*tun.NativeTun)

	actualInterfaceName, err := nativeTun.Name()
	if err != nil {
		nativeTun.Close()
		logger.Error.Println("Failed to determine name of wintun adapter")
		return ERROR_GENERAL_FAILURE
	}
	if actualInterfaceName != ifaceName {
		// WireGuard picked a different name for the adapter than the one we expected.
		// This indicates there is already an adapter with the name we intended to use.
		logger.Debug.Println("Failed to create adapter with specific name")
	}

	device := device.NewDevice(wintun, logger)

	setError := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		logger.Error.Println("Failed to set device configuration")
		logger.Error.Println(setError)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	device.Up()

	interfaces := createInterfaceWatcherEvents(waitOnIpv6, nativeTun.LUID())

	logger.Debug.Println("Waiting for interfaces to attach")

	if !watcher.Join(interfaces, 5) {
		logger.Error.Println("Failed to wait for IP interfaces to become available")
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	logger.Debug.Println("Interfaces OK")

	context := tunnelcontainer.Context{
		Device: device,
		Logger: logger,
	}

	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Error.Println(err)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	if cIfaceNameOut != nil {
		*cIfaceNameOut = C.CString(actualInterfaceName)
	}

	return handle
}

//export wgRebindTunnelSocket
func wgRebindTunnelSocket(family uint16, interfaceIndex uint32) {
	tunnels.ForEach(func(tunnel tunnelcontainer.Context) {
		blackhole := (interfaceIndex == 0)
		bind := tunnel.Device.Bind().(conn.BindSocketToInterface)

		if family == windows.AF_INET {
			tunnel.Logger.Info.Printf("Binding v4 socket to interface %d (blackhole=%v)", interfaceIndex, blackhole)
			err := bind.BindSocketToInterface4(interfaceIndex, blackhole)
			if err != nil {
				tunnel.Logger.Info.Println(err)
			}
		} else if family == windows.AF_INET6 {
			tunnel.Logger.Info.Printf("Binding v6 socket to interface %d (blackhole=%v)", interfaceIndex, blackhole)
			err := bind.BindSocketToInterface6(interfaceIndex, blackhole)
			if err != nil {
				tunnel.Logger.Info.Println(err)
			}
		}
	})
}
