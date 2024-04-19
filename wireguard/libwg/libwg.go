/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdio.h>
// #include <stdlib.h>
// #include "libwg.h"
// #include "../../libmaybenot/libmaybenot.h"
//
// void wgOnMaybenotAction(void* tunnelHandle, MaybenotAction action);
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

const (
	ERROR_GENERAL_FAILURE      = -1
	ERROR_INTERMITTENT_FAILURE = -2
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
	}
	// Calling twice convinces the GC to release NOW.
	runtime.GC()
	runtime.GC()
}

//export wgActivateDaita
func wgActivateDaita(machines *C.int8_t, tunnelHandle int32, eventsCapacity uint32, actionsCapacity uint32) C.bool {
	tunnel, err := tunnels.Get(tunnelHandle)
	if err != nil {
		return false
	}

	tunnel.Logger.Errorf("Initializing libmaybenot")
	var maybenot *C.Maybenot
	maybenot_result := C.maybenot_start(
		machines, 0.0, 0.0, 1440,
		&maybenot,
	)

	if maybenot == nil {
		tunnel.Logger.Errorf("Failed to initialize maybenot: %d", maybenot_result)
		return false
	}
	tunnel.Logger.Errorf("Success :eyes:")

	if !tunnel.Device.ActivateDaita(uint(eventsCapacity), uint(actionsCapacity)) {
		C.maybenot_stop(maybenot)
		return false
	}

	go handleEvents(maybenot, tunnelHandle, tunnel)
	return true
}

func handleEvents(maybenot *C.Maybenot, tunnelHandle int32, tunnel tunnelcontainer.Context) int32 {
	if tunnel.Device == nil {
		tunnel.Logger.Errorf("No device for tunnel?")
		return -1
	}

	if tunnel.Device.Daita == nil {
		tunnel.Logger.Errorf("DAITA not activated")
		return -2
	}

	for {
		event := tunnel.Device.Daita.ReceiveEvent()
		if event == nil {
			tunnel.Logger.Errorf("No more DAITA events")
			C.maybenot_stop(maybenot)
			return 0
		}

		cEvent := C.MaybenotEvent{
			machine:    0, // TODO
			event_type: C.uint32_t(event.EventType),
			xmit_bytes: C.uint16_t(event.XmitBytes),
		}

		// TODO: is unsafe.Pointer sound?
		var num_actions C.uint64_t
		actions := make([]C.MaybenotAction, 10) // TODO: Actually use a valid capacity here
		C.maybenot_on_event(maybenot, cEvent, &actions[0], &num_actions)
	}
}

func wgOnMaybenotAction(userData *C.void, action C.MaybenotAction) {
	// TODO: is this safe? will go try to garbage collect this pointer? what happens if i leak it?
	ctx := (*EventContext)(unsafe.Pointer(userData))

	tunnel, err := tunnels.Get(ctx.tunnelHandle)
	if err != nil {
		// Failed to get tunnel from handle, cannot log
		return
	}

	if tunnel.Device.Daita == nil {
		tunnel.Logger.Errorf("DAITA not activated")
		return
	}

	// TODO: support more actions
	if action.tag != 1 /* INJECT_PADDING */ {
		tunnel.Logger.Errorf("Got non-padding action")
		return
	}

	// cast union to the ActionInjectPadding variant
	padding_action := (*C.MaybenotAction_InjectPadding_Body)(unsafe.Pointer(&action.anon0[0]))

	action_go := device.Action{
		Peer:       ctx.peer,
		ActionType: 1,
		Payload: device.Padding{
			ByteCount: uint16(padding_action.size),
			Replace:   bool(padding_action.replace),
		},
	}

	err = tunnel.Device.Daita.SendAction(action_go)
	if err != nil {
		tunnel.Logger.Errorf("Failed to send DAITA action %v because of %v", action_go, err)
		return
	}
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
