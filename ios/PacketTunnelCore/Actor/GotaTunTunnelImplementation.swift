//
//  GotaTunTunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging

/// GotaTun tunnel implementation.
/// Unlike WireGuardGo, this implementation does NOT use an external state observer.
/// State transitions are handled internally by the GotaTun actor.
public final class GotaTunTunnelImplementation: TunnelImplementation, @unchecked Sendable {
    private let logger = Logger(label: "GotaTunTunnelImplementation")

    private let _actor = GotaTunActor()
    public var actor: any PacketTunnelActorProtocol { _actor }

    public init() {}

    public func startTunnel(options: StartOptions) async {
        // NO startObservingActorState() - this is the key architectural difference.
        // The GotaTun actor handles state internally.
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
