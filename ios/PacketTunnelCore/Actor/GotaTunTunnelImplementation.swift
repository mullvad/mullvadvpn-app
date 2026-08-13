//
//  GotaTunTunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes

/// GotaTun tunnel implementation.
/// Unlike WireGuardGo, this implementation does NOT use an external state observer.
/// GotaTunActor may be used directly by the PacketTunnelProvider once we get rid of WireGuardGo.
public final class GotaTunTunnelImplementation: TunnelImplementation, Sendable {
    private let gotaTunActor: GotaTunActor
    public var actor: any PacketTunnelActorProtocol { gotaTunActor }

    public init(
        providerDelegate: sending TunnelProviderDelegate,
        blockedStateErrorMapper: sending BlockedStateErrorMapperProtocol,
        adapterFactory: GotaTunAdapterFactory,
        ipOverrideWrapper: IPOverrideWrapper,
        settingsReader: sending TunnelSettingsManager
    ) {
        gotaTunActor = GotaTunActor(
            providerDelegate: providerDelegate,
            settingsReader: settingsReader,
            relaySelector: RelaySelectorWrapper(relayCache: ipOverrideWrapper),
            blockedStateErrorMapper: blockedStateErrorMapper,
            adapterFactory: adapterFactory
        )
    }

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
        gotaTunActor.onWake()
    }
}
