// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging

/// GotaTun tunnel implementation. Once WireGuardGo is gone, PacketTunnelProvider can interact with GotaTunActor directly. This component is here only to bridge the gap whilst both are available
public final class GotaTunTunnelImplementation: TunnelImplementation, Sendable {
    private let _actor = GotaTunActor()
    public var actor: any PacketTunnelActorProtocol { _actor }

    public init() {}

    public func startTunnel(options: StartOptions) async {
        await actor.start(options: options)
    }

    public func stopTunnel() async {
        await actor.stop()
        await actor.waitUntilDisconnected()
    }

    public func sleep() async {
        await actor.onSleep()
    }

    public func wake() {
        actor.onWake()
    }
}
