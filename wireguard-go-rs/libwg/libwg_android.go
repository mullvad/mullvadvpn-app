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
	"errors"
	"net/netip"
	"strings"
	"unsafe"

	"golang.org/x/sys/unix"

	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun"
	"golang.zx2c4.com/wireguard/tun/multihoptun"

	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/logging"
	"github.com/mullvad/mullvadvpn-app/wireguard/libwg/tunnelcontainer"
)

// Redefined here because otherwise the compiler doesn't realize it's a type alias for a type that's safe to export.
// Taken from the contained logging package.
type LogSink = unsafe.Pointer
type LogContext = C.uint64_t

// TODO: Document
type tunnelHandle struct {
	exit   *device.Device
	entry  *device.Device
	logger *device.Logger
}

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

//export wgTurnOnMultihop
func wgTurnOnMultihop(cExitSettings *C.char, cEntrySettings *C.char, privateIp *C.char, fd int, logSink LogSink, logContext LogContext) C.int32_t {
	logger := logging.NewLogger(logSink, logging.LogContext(logContext))
	if cExitSettings == nil {
		logger.Errorf("cExitSettings is null\n")
		return ERROR_INVALID_ARGUMENT
	}
	exitSettings := goStringFixed(cExitSettings)

	if cEntrySettings == nil {
		logger.Errorf("cEntrySettings is null\n")
		return ERROR_INVALID_ARGUMENT
	}
	entrySettings := goStringFixed(cEntrySettings)

	exitEndpoint := parseEndpointFromConfig(exitSettings)

	if exitEndpoint == nil {
		logger.Errorf("exitEndpoint is null\n")
		return ERROR_INVALID_ARGUMENT
	}

	// Set up a two tunnel devices: One 'fake' device for the exit relay and one 'real' device for the entry relay

	tunDevice, _, err := tun.CreateUnmonitoredTUNFromFD(fd)
	if err != nil {
		logger.Errorf("%s\n", err)
		unix.Close(fd)
		if err.Error() == "bad file descriptor" {
			return ERROR_INTERMITTENT_FAILURE
		}
		return ERROR_GENERAL_FAILURE
	}

	ip, err := netip.ParseAddr(goStringFixed(privateIp))
	if err != nil {
		logger.Errorf("%s\n", err)
		tunDevice.Close()
		return ERROR_INVALID_ARGUMENT
	}

	mtu, err := tunDevice.MTU()
	if err != nil {
		logger.Errorf("%s\n", err)
		tunDevice.Close()
		return ERROR_GENERAL_FAILURE
	}

	singleTunMtu := mtu - 80 //Internet mtu - Wireguard header size - ipv4 UDP header
	singletun := multihoptun.NewMultihopTun(ip, exitEndpoint.Addr(), exitEndpoint.Port(), singleTunMtu)

	entryDevice := device.NewDevice(tunDevice, singletun.Binder(), logger)

	setErr := entryDevice.IpcSetOperation(bufio.NewReader(strings.NewReader(entrySettings)))
	if setErr != nil {
		logger.Errorf("%s\n", setErr)
		entryDevice.Close()
		return ERROR_INTERMITTENT_FAILURE
	}

	entryDevice.DisableSomeRoamingForBrokenMobileSemantics()

	exitDevice := device.NewDevice(&singletun, conn.NewStdNetBind(), logger)

	setErr = exitDevice.IpcSetOperation(bufio.NewReader(strings.NewReader(exitSettings)))
	if setErr != nil {
		logger.Errorf("%s\n", setErr)
		exitDevice.Close()
		return ERROR_INTERMITTENT_FAILURE
	}

	exitDevice.DisableSomeRoamingForBrokenMobileSemantics()

	exitDevice.Up()
	entryDevice.Up()

	// Create the stuff that needs

	context := tunnelcontainer.Context{
		Device: exitDevice,
		EntryDevice: entryDevice,
		Logger: logger,
	}

	handle, err := tunnels.Insert(context)
	if err != nil {
		logger.Errorf("%s\n", err)
		entryDevice.Close()
		exitDevice.Close()
		return ERROR_GENERAL_FAILURE
	}

	return C.int32_t(handle)

}

//export wgTurnOnMultihop
/*func wgTurnOnMultihop(mtu int, exitSettings *C.char, entrySettings *C.char, privateIp *C.char, fd int32, logSink LogSink, logContext LogContext) C.int32_t {
	logger := logging.NewLogger(logSink, logging.LogContext(logContext))

	if exitSettings == nil {
		logger.Errorf("exitSettings is null\n")
		return ERROR_INVALID_ARGUMENT
	}

	if entrySettings == nil {
		logger.Errorf("exitSettings is null\n")
		return ERROR_INVALID_ARGUMENT
	}

	// OLD
	// settings := goStringFixed(cSettings)
	// NEW
	exitConfigString := goStringFixed(exitSettings)
	entryConfigString := goStringFixed(entrySettings)
	exitEndpoint := parseEndpointFromConfig(exitConfigString)
	if exitEndpoint == nil {
		return -10 // TODO: Define this error, previously '' errNoEndpointInConfig
	}

	ip, err := netip.ParseAddr(goStringFixed(privateIp))
	if err != nil {
		logger.Errorf("Failed to parse private IP: %v", err)
		return -10 // TODO: Define this error, previously '' errBadIPString
	}

	// OLD
	// device := device.NewDevice(tunDevice, conn.NewStdNetBind(), logger)
	// NEW
	// ip: First hop
	// exitEndpoint: Last hop
	// TODO: Is this mtu the correct one?
	singleTunMtu := mtu - 80
	singletun := multihoptun.NewMultihopTun(ip, exitEndpoint.Addr(), exitEndpoint.Port(), singleTunMtu)
	entryDev := device.NewDevice(&singletun, conn.NewStdNetBind(), logger)

	tunDevice, _, err := tun.CreateUnmonitoredTUNFromFD(fd)
	if err != nil {
		logger.Errorf("%s\n", err)
		unix.Close(fd)
		if err.Error() == "bad file descriptor" {
			return ERROR_INTERMITTENT_FAILURE
		}
		return ERROR_GENERAL_FAILURE
	}
	exitDev := device.NewDevice(tunDevice, singletun.Binder(), logger)

	setErr := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setErr != nil {
		logger.Errorf("%s\n", setErr)
		device.Close()
		return ERROR_INTERMITTENT_FAILURE
	}

	device.DisableSomeRoamingForBrokenMobileSemantics()
	device.Up()

	// context := tunnelcontainer.Context{
	// 	Device: device,
	// 	Logger: logger,
	// }

	handle, err := addTunnelFromDevice(exitDev, entryDev, exitSettings, entrySettings, logger)
	if err != nil {
		logger.Errorf("%s\n", err)
		device.Close()
		return ERROR_GENERAL_FAILURE
	}

	return C.int32_t(handle)
}*/

func addTunnelFromDevice(exitDev *device.Device, entryDev *device.Device, exitSettings string, entrySettings string, logger *device.Logger) (*tunnelHandle, error) {
	err := bringUpDevice(exitDev, exitSettings, logger)
	if err != nil {
		return nil, errors.New("Could not bring up exit device") // errBadWgConfig
	}

	if entryDev != nil {
		err = bringUpDevice(entryDev, entrySettings, logger)
		if err != nil {
			exitDev.Close()
			return nil, errors.New("Could not bring up entry device")
		}
	}

	return &tunnelHandle{exitDev, entryDev, logger}, nil
}

func bringUpDevice(dev *device.Device, settings string, logger *device.Logger) error {
	err := dev.IpcSet(settings)
	if err != nil {
		logger.Errorf("Unable to set IPC settings: %v", err)
		dev.Close()
		return err
	}

	dev.Up()
	logger.Verbosef("Device started")
	return nil
}

// Parse a wireguard config and return the first endpoint address it finds and
// parses successfully.gi b
func parseEndpointFromConfig(config string) *netip.AddrPort {
	scanner := bufio.NewScanner(strings.NewReader(config))
	for scanner.Scan() {
		line := scanner.Text()
		key, value, ok := strings.Cut(line, "=")
		if !ok {
			continue
		}

		if key == "endpoint" {
			endpoint, err := netip.ParseAddrPort(value)
			if err == nil {
				return &endpoint
			}
		}

	}
	return nil
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
