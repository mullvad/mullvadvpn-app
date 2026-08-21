// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

/// Protocol for tunnel backend implementations.
/// The picker (`PacketTunnelProvider`) delegates all lifecycle events to the active implementation.
public protocol TunnelImplementation: AnyObject {
    /// The underlying actor that handles tunnel state.
    var actor: any PacketTunnelActorProtocol { get }

    /// Called when the tunnel is starting, after initial network settings have been applied.
    func startTunnel(options: StartOptions) async

    /// Called when the tunnel is stopping. Must wait until disconnected before returning.
    func stopTunnel() async

    /// Called when the device is going to sleep.
    func sleep() async

    /// Called when the device wakes up.
    func wake()
}
