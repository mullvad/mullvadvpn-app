//
//  GotaTunTunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
