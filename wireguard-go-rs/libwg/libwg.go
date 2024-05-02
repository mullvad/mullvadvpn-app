/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 */

package main

// #include <stdio.h>
// #include <stdlib.h>
// #include "libwg.h"
// #include "../cmaybenot/libmaybenot.h"
import "C"

import (
	"bufio"
	"bytes"
	"runtime"
	"strings"
	"time"
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

	tunnel.Logger.Verbosef("Initializing libmaybenot")
	var maybenot *C.Maybenot
	maybenot_result := C.maybenot_start(
		machines, 0.0, 0.0, 1440,
		&maybenot,
	)

	if maybenot_result != 0 {
		tunnel.Logger.Errorf("Failed to initialize maybenot, code=%d", maybenot_result)
		return false
	}

	if !tunnel.Device.ActivateDaita(uint(eventsCapacity), uint(actionsCapacity)) {
		tunnel.Logger.Errorf("Failed to activate DAITA")
		C.maybenot_stop(maybenot)
		return false
	}

	numMachines := C.maybenot_num_machines(maybenot)
	daita := DaitaThingy{
		tunnel:         tunnel,
		tunnelHandle:   tunnelHandle,
		maybenot:       maybenot,
		newActionsBuf:  make([]C.MaybenotAction, numMachines),
		machineActions: map[uint64]device.Action{},
	}
	go daita.handleEvents()
	return true
}

type DaitaThingy struct {
	tunnel         tunnelcontainer.Context
	tunnelHandle   int32
	maybenot       *C.Maybenot
	newActionsBuf  []C.MaybenotAction
	machineActions map[uint64]device.Action
}

func (self DaitaThingy) handleEvents() {
	if self.tunnel.Device == nil {
		self.tunnel.Logger.Errorf("No device for tunnel?")
		return
	}

	if self.tunnel.Device.Daita == nil {
		self.tunnel.Logger.Errorf("DAITA not activated")
		return
	}

	// TODO: proper race-condition safe nil checks for everything
	events := self.tunnel.Device.Daita.Events

	// create a new inactive timer to help us track when maybenot actions should be performed.
	actionTimer := time.NewTimer(time.Duration(999999999999999999)) // wtf
	actionTimer.Stop()

	for {
		now := time.Now()

		// get the time until the next action from machineActions should be performed, if any
		var nextActionMachine *uint64 = nil
		var nextActionIn time.Duration
		for machine, action := range self.machineActions {
			timeUntilAction := action.Time.Sub(now)

			if nextActionMachine == nil || timeUntilAction < nextActionIn {
				nextActionIn = timeUntilAction
				nextActionMachine = &machine
			}
		}

		// if we found a pending action, set the timer
		if nextActionMachine != nil {
			actionTimer.Reset(nextActionIn)
		}

		// wait until we either get a new event, or until an action is supposed to fire
		select {
		case event := <-events:
			// make sure the timer is stopped and cleared
			if nextActionMachine != nil && !actionTimer.Stop() {
				<-actionTimer.C
			}

			if event == nil {
				self.tunnel.Logger.Errorf("No more DAITA events")
				C.maybenot_stop(self.maybenot)
				return
			}

			self.handleEvent(*event)

		case <-actionTimer.C:
			// it's time to do the action! pop it from the map and send it to wireguard-go
			action := self.machineActions[*nextActionMachine]
			delete(self.machineActions, *nextActionMachine)
			self.actOnAction(action)
		}
	}
}

func (self DaitaThingy) handleEvent(event device.Event) {
	cEvent := C.MaybenotEvent{
		machine:    C.uint64_t(event.Machine),
		event_type: C.uint32_t(event.EventType),
		xmit_bytes: C.uint16_t(event.XmitBytes),
	}

	var actionsWritten C.uint64_t

	// TODO: is it even sound to pass a slice reference like this?
	// TODO: handle error
	C.maybenot_on_event(self.maybenot, cEvent, &self.newActionsBuf[0], &actionsWritten)

	// TODO: there is a small disparity here, between the time used by maybenot_on_event,
	// and `now`. Is this a problem?
	now := time.Now()

	newActions := self.newActionsBuf[0:actionsWritten]
	for _, newAction := range newActions {
		// TODO: support more actions
		if newAction.tag != 1 /* INJECT_PADDING */ {
			self.tunnel.Logger.Errorf("ignoring action type %d, unimplemented", newAction.tag)
			continue
		}

		newActionGo := self.maybenotActionToGo(newAction, now, event.Peer)
		machine := newActionGo.Machine
		self.machineActions[machine] = newActionGo
	}
}

func (self DaitaThingy) maybenotActionToGo(action_c C.MaybenotAction, now time.Time, peer device.NoisePublicKey) device.Action {
	// TODO: support more actions
	if action_c.tag != 1 /* INJECT_PADDING */ {
		// panic!
	}

	// cast union to the ActionInjectPadding variant
	padding_action := (*C.MaybenotAction_InjectPadding_Body)(unsafe.Pointer(&action_c.anon0[0]))

	timeout := maybenotDurationToGoDuration(padding_action.timeout)

	return device.Action{
		Peer:       peer,
		Machine:    uint64(padding_action.machine),
		Time:       now.Add(timeout),
		ActionType: 1, // TODO
		Payload: device.Padding{
			ByteCount: uint16(padding_action.size),
			Replace:   bool(padding_action.replace),
		},
	}
}

func maybenotDurationToGoDuration(duration C.MaybenotDuration) time.Duration {
	// let's just assume this is fine...
	nanoseconds := uint64(duration.secs)*1_000_000_000 + uint64(duration.nanos)
	return time.Duration(nanoseconds)
}

func (self DaitaThingy) actOnAction(action device.Action) {
	err := self.tunnel.Device.Daita.SendAction(action)
	if err != nil {
		self.tunnel.Logger.Errorf("Failed to send DAITA action %v because of %v", action, err)
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
