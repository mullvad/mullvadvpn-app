//
//  GotaTunTunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
@preconcurrency import NetworkExtension

/// GotaTun tunnel implementation.
/// Unlike WireGuardGo, this implementation does NOT use an external state observer.
/// State transitions are handled internally by the GotaTun actor.
///
/// Concrete dependencies that require the `PacketTunnel` target (tunnel FD,
/// network settings, path observer) are injected via `GotaTunProviderDelegate`.
public final class GotaTunTunnelImplementation: TunnelImplementation, @unchecked Sendable {
    private let logger = Logger(label: "GotaTunTunnelImplementation")

    private var _actor: GotaTunActor!
    public var actor: any PacketTunnelActorProtocol { _actor }

    private let providerDelegate: GotaTunProviderDelegate
    private let blockedStateErrorMapper: BlockedStateErrorMapperProtocol
    private let adapterFactory: GotaTunAdapterFactory

    public init(
        providerDelegate: GotaTunProviderDelegate,
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol,
        adapterFactory: GotaTunAdapterFactory
    ) {
        self.providerDelegate = providerDelegate
        self.blockedStateErrorMapper = blockedStateErrorMapper
        self.adapterFactory = adapterFactory
    }

    public func setUp(
        provider: NEPacketTunnelProvider,
        internalQueue: DispatchQueue,
        ipOverrideWrapper: IPOverrideWrapper,
        settingsReader: sending TunnelSettingsManager,
        apiTransportProvider: APITransportProvider
    ) {
        let defaultPathObserver = providerDelegate.makeDefaultPathObserver(eventQueue: internalQueue)
        let relaySelector = RelaySelectorWrapper(relayCache: ipOverrideWrapper)

        _actor = GotaTunActor(
            tunnelFd: { [providerDelegate] in providerDelegate.tunnelFileDescriptor },
            applyNetworkSettings: { [providerDelegate] settings in
                try await providerDelegate.applyNetworkSettings(settings)
            },
            settingsReader: settingsReader,
            relaySelector: relaySelector,
            defaultPathObserver: defaultPathObserver,
            blockedStateErrorMapper: blockedStateErrorMapper,
            adapterFactory: adapterFactory
        )
    }

    public func startTunnel(options: StartOptions) async {
        actor.start(options: options)
    }

    public func stopTunnel() async {
        actor.stop()
        await actor.waitUntilDisconnected()
    }

    public func sleep() async {
        actor.onSleep()
    }

    public func wake() {
        actor.onWake()
    }
}
