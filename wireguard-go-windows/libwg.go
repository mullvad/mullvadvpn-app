/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2019 Amagicom AB. All Rights Reserved.
 */

package main

// #include <stdlib.h>
// #include <sys/types.h>
// typedef void (__stdcall *LogSink)(unsigned int, const char *, void *);
// static void callLogSink(void *logSink, int level, const char *message, void *context)
// {
//   ((LogSink)logSink)((unsigned int)level, message, context);
// }
import "C"

import (
	"bufio"
	"bytes"
	"errors"
	"log"
	"math"
	"net"
	"runtime"
	"strings"
	"unsafe"

	"golang.org/x/sys/windows"

	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/ipc"
	"golang.zx2c4.com/wireguard/tun"
	"golang.zx2c4.com/wireguard/windows/tunnel/winipcfg"
)

// Define type aliases.
type LogSink = unsafe.Pointer
type LogContext = unsafe.Pointer

type Logger struct {
	sink    LogSink
	context LogContext
	level   C.int
}

func (l *Logger) Write(message []byte) (int, error) {
	msg := C.CString(string(message))
	C.callLogSink(l.sink, l.level, msg, l.context)
	C.free(unsafe.Pointer(msg))
	return len(message), nil
}

type TunnelContext struct {
	device *device.Device
	uapi   net.Listener
	logger *device.Logger
}

var tunnels map[int32]TunnelContext

func init() {
	device.RoamingDisabled = true
	tunnels = make(map[int32]TunnelContext)
}

func newLogger(logSink LogSink, logContext LogContext) *device.Logger {
	logger := new(device.Logger)

	logger.Debug = log.New(
		&Logger{sink: logSink, context: logContext, level: device.LogLevelDebug},
		"",
		0,
	)
	logger.Info = log.New(
		&Logger{sink: logSink, context: logContext, level: device.LogLevelInfo},
		"",
		0,
	)
	logger.Error = log.New(
		&Logger{sink: logSink, context: logContext, level: device.LogLevelError},
		"",
		0,
	)

	return logger
}

// Find next free context slot
func getContextHandle() (int32, error) {
	var i int32
	for i = 0; i < math.MaxInt32; i++ {
		if _, exists := tunnels[i]; !exists {
			break
		}
	}

	if i == math.MaxInt32 {
		return 0, errors.New("Handle table is full")
	}

	return i, nil
}

//export wgTurnOn
func wgTurnOn(cIfaceName *C.char, mtu int, cSettings *C.char, logSink LogSink, logContext LogContext) int32 {
	logger := newLogger(logSink, logContext)

	if cIfaceName == nil {
		logger.Error.Println("cIfaceName is null")
		return -1
	}

	if cSettings == nil {
		logger.Error.Println("cSettings is null")
		return -1
	}

	contextHandle, err := getContextHandle()
	if err != nil {
		logger.Error.Println(err)
		return -1
	}

	settings := C.GoString(cSettings)
	ifaceName := C.GoString(cIfaceName)

	// {AFE43773-E1F8-4EBB-8536-576AB86AFE9A}
	networkId := windows.GUID { 0xafe43773, 0xe1f8, 0x4ebb, [8]byte{ 0x85, 0x36, 0x57, 0x6a, 0xb8, 0x6a, 0xfe, 0x9a } }

	watcher, err := watchInterfaces()
	if err != nil {
		logger.Error.Println(err)
		return -1
	}
	defer watcher.destroy()

	wintun, err := tun.CreateTUNWithRequestedGUID(ifaceName, &networkId, mtu)
	if err != nil {
		logger.Error.Println("Failed to create tunnel")
		logger.Error.Println(err)
		return -1
	}

	nativeTun := wintun.(*tun.NativeTun)

	actualInterfaceName, err := nativeTun.Name()
	if err != nil {
		nativeTun.Close()
		logger.Error.Println("Failed to determine name of wintun adapter")
		return -1
	}

	if actualInterfaceName != ifaceName {
		// WireGuard picked a different name for the adapter than the one we expected.
		// This indicates there is already an adapter with the name we intended to use.
		nativeTun.Close()
		logger.Error.Println("Failed to create adapter with specific name")
		return -1
	}

	device := device.NewDevice(wintun, logger)

	uapi, err := ipc.UAPIListen(ifaceName)
	if err != nil {
		logger.Error.Println("Failed to start UAPI")
		logger.Error.Println(err)
		device.Close()
		return -1
	}

	setError := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		logger.Error.Println("Failed to set device configuration")
		logger.Error.Println(setError)
		uapi.Close()
		device.Close()
		return -1
	}

	device.Up()

	interfaces := []interfaceWatcherEvent{
		{
			luid: winipcfg.LUID(nativeTun.LUID()),
			family: windows.AF_INET,
		},
		{
			luid: winipcfg.LUID(nativeTun.LUID()),
			family: windows.AF_INET6,
		},
	}

	logger.Debug.Println("Waiting for interfaces to attach")

	if !watcher.join(interfaces, 5) {
		logger.Error.Println("Failed to wait for IP interfaces to become available")
		uapi.Close()
		device.Close()
		return -1
	}

	logger.Debug.Println("Interfaces OK")

	// Service UAPI.
	go func() {
		for {
			conn, err := uapi.Accept()
			if err != nil {
				logger.Info.Println("UAPI Accept() failed")
				logger.Info.Println(err)
				return
			}
			go device.IpcHandle(conn)
		}
	}()

	tunnels[contextHandle] = TunnelContext{
		device: device,
		uapi: uapi,
		logger: logger,
	}

	return contextHandle
}

//export wgTurnOff
func wgTurnOff(contextHandle int32) {
	{
		context, ok := tunnels[contextHandle]
		if !ok {
			return
		}
		delete(tunnels, contextHandle)
		context.uapi.Close()
		context.device.Close()
	}
	// Calling twice convinces the GC to release NOW.
	runtime.GC()
	runtime.GC()
}

//export wgRebindTunnelSocket
func wgRebindTunnelSocket(family uint16, interfaceIndex uint32) {
	for _, tunnel := range tunnels {
		blackhole := (interfaceIndex == 0)
		if family == windows.AF_INET {
			tunnel.logger.Info.Printf("Binding v4 socket to interface %d (blackhole=%v)", interfaceIndex, blackhole)
			err := tunnel.device.BindSocketToInterface4(interfaceIndex, blackhole)
			if err != nil {
				tunnel.logger.Info.Println(err)
			}
		} else if family == windows.AF_INET6 {
			tunnel.logger.Info.Printf("Binding v6 socket to interface %d (blackhole=%v)", interfaceIndex, blackhole)
			err := tunnel.device.BindSocketToInterface6(interfaceIndex, blackhole)
			if err != nil {
				tunnel.logger.Info.Println(err)
			}
		}
	}
}

//export wgVersion
func wgVersion() *C.char {
	return C.CString(device.WireGuardGoVersion)
}

//export wgGetConfig
func wgGetConfig(tunnelHandle int32) *C.char {
	tunnel, ok := tunnels[tunnelHandle]
	if !ok {
		return nil
	}

	settings := new(bytes.Buffer)
	writer := bufio.NewWriter(settings)
	if err := tunnel.device.IpcGetOperation(writer); err != nil {
		tunnel.logger.Error.Println("Failed to get config for tunnel: ", err)
		return nil
	}
	writer.Flush()
	return C.CString(settings.String())
}

//export wgFreePtr
func wgFreePtr(ptr unsafe.Pointer) {
	C.free(ptr)
}

func main() {}
