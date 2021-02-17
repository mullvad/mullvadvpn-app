/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2019 WireGuard LLC. All Rights Reserved.
 * Copyright (C) 2021 Mullvad VPN AB. All Rights Reserved.
 */

package interfacewatcher

import (
	"sync"
	"time"
	
	"golang.zx2c4.com/wireguard/windows/tunnel/winipcfg"
)

type Event struct {
	Luid   winipcfg.LUID
	Family winipcfg.AddressFamily
}

type interfaceWatcher struct {
	ready                   chan bool
	processingMutex         sync.Mutex
	interfaceChangeCallback *winipcfg.InterfaceChangeCallback
	seenEvents              []Event
	wantedEvents			[]Event
	expired                 bool
}

func NewWatcher() (*interfaceWatcher, error) {
	iw := &interfaceWatcher{
		ready: make(chan bool, 1),
		expired: false,
	}
	var err error
	iw.interfaceChangeCallback, err = winipcfg.RegisterInterfaceChangeCallback(func(notificationType winipcfg.MibNotificationType, iface *winipcfg.MibIPInterfaceRow) {
		if notificationType != winipcfg.MibAddInstance {
			return
		}

		iw.processingMutex.Lock()
		defer iw.processingMutex.Unlock()

		if iw.expired {
			return
		}

		iw.seenEvents = append(iw.seenEvents, Event{iface.InterfaceLUID, iface.Family})

		if len(iw.wantedEvents) != 0 {
			iw.evaluateEvents()
		}
	})
	if err != nil {
		return nil, err
	}
	return iw, nil
}

func (iw *interfaceWatcher) evaluateEvents() {
	matched := 0

	// This is n*n, but typically very few items in both slices :-)
	for _, wanted := range iw.wantedEvents {
		for _, seen := range iw.seenEvents {
			if seen == wanted {
				matched += 1
				break
			}
		}
	}

	if len(iw.wantedEvents) != matched {
		return
	}

	iw.expired = true
	iw.ready <- true
}

// You can only join() once after which the watcher becomes expired.
func (iw *interfaceWatcher) Join(wantedEvents []Event, timeoutSeconds int) bool {
	{
		iw.processingMutex.Lock()

		if iw.expired || len(wantedEvents) == 0 {
			iw.processingMutex.Unlock()
			return false
		}

		iw.wantedEvents = wantedEvents
		iw.evaluateEvents()

		iw.processingMutex.Unlock()
	}

	result := false

	select {
    case <- iw.ready:
        result = true
    case <- time.After(time.Duration(timeoutSeconds) * time.Second):
        result = false
	}

	{
		iw.processingMutex.Lock()

		iw.wantedEvents = nil
		iw.expired = true

		iw.processingMutex.Unlock()
	}

	return result
}

func (iw *interfaceWatcher) Destroy() {
	if iw.interfaceChangeCallback != nil {
		iw.interfaceChangeCallback.Unregister()
		iw.interfaceChangeCallback = nil
	}
}
