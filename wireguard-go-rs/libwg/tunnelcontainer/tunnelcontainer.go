/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2020 Mullvad VPN AB. All Rights Reserved.
 */

package tunnelcontainer

import (
	"net"
	"math"
	"errors"

	"golang.zx2c4.com/wireguard/device"
)

type Context struct {
	Device *device.Device
	Uapi   net.Listener
	Logger *device.Logger
}

type Container struct {
	tunnels map[int32]Context
}

func New() Container {
	return Container{
		tunnels: make(map[int32]Context),
	}
}

func (tc *Container) Insert(context Context) (int32, error) {
	var i int32
	for i = 0; i < math.MaxInt32; i++ {
		if _, exists := tc.tunnels[i]; !exists {
			break
		}
	}

	if i == math.MaxInt32 {
		return 0, errors.New("Container is full")
	}

	tc.tunnels[i] = context
	return i, nil
}

func (tc *Container) Get(handle int32) (Context, error){
	context, ok := tc.tunnels[handle]
	if !ok {
		return Context{}, errors.New("Invalid context handle")
	}
	return context, nil
}

func (tc *Container) Remove(handle int32) (Context, error){
	context, ok := tc.tunnels[handle]
	if !ok {
		return Context{}, errors.New("Invalid context handle")
	}
	delete(tc.tunnels, handle)
	return context, nil
}

func (tc *Container) ForEach(callback func(Context)) {
	for _, tunnel := range tc.tunnels {
		callback(tunnel)
	}
}
