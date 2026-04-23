//
//  WireGuardGoTunnelImplementation.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
@preconcurrency import NetworkExtension
import PacketTunnelCore

/// WireGuardGo tunnel implementation.
/// Owns the WgAdapter, TunnelMonitor, state observer, path observer,
/// and ephemeral peer exchange pipeline.
final class WireGuardGoTunnelImplementation: TunnelImplementation, @unchecked Sendable {
    private let logger = Logger(label: "WireGuardGoTunnelImplementation")
    private weak var provider: NEPacketTunnelProvider?

    // WG-specific infrastructure
    private var adapter: WgAdapter!
    private var relaySelector: RelaySelectorWrapper!
    private var ephemeralPeerExchangingPipeline: EphemeralPeerExchangingPipeline!
    private var stateObserverTask: AnyTask?
    private var defaultPathObserver: PacketTunnelPathObserver!
    private lazy var ephemeralPeerReceiver: EphemeralPeerReceiver = {
        EphemeralPeerReceiver(tunnelProvider: adapter, keyReceiver: self)
    }()

    private var _actor: PacketTunnelActor!
    var actor: any PacketTunnelActorProtocol { _actor }

    /// Callback to trigger a device check from the picker (shared infrastructure).
    var onDeviceCheck: (() -> Void)?

    func setUp(
        provider: NEPacketTunnelProvider,
        internalQueue: DispatchQueue,
        ipOverrideWrapper: IPOverrideWrapper,
        settingsReader: sending TunnelSettingsManager,
        apiTransportProvider: APITransportProvider
    ) {
        self.provider = provider

        defaultPathObserver = PacketTunnelPathObserver(eventQueue: internalQueue)

        adapter = WgAdapter(packetTunnelProvider: provider)

        let pinger = TunnelPinger(pingProvider: adapter.icmpPingProvider, replyQueue: internalQueue)

        let tunnelMonitor = TunnelMonitor(
            eventQueue: internalQueue,
            pinger: pinger,
            tunnelDeviceInfo: adapter,
            timings: TunnelMonitorTimings()
        )

        relaySelector = RelaySelectorWrapper(
            relayCache: ipOverrideWrapper
        )

        _actor = PacketTunnelActor(
            timings: PacketTunnelActorTimings(),
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: defaultPathObserver,
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: relaySelector,
            settingsReader: settingsReader,
            protocolObfuscator: ProtocolObfuscator<TunnelObfuscator>()
        )

        // Since PacketTunnelActor depends on the path observer, start observing after actor has been initalized.
        startDefaultPathObserver()

        ephemeralPeerExchangingPipeline = EphemeralPeerExchangingPipeline(
            EphemeralPeerExchangeActor(
                packetTunnel: ephemeralPeerReceiver,
                onFailure: self.ephemeralPeerExchangeFailed,
                iteratorProvider: { REST.RetryStrategy.postQuantumKeyExchange.makeDelayIterator() }
            ),
            onUpdateConfiguration: { [weak self] configuration in
                guard let self else { return }
                let channel = OneshotChannel()
                _actor.changeEphemeralPeerNegotiationState(
                    configuration: configuration,
                    reconfigurationSemaphore: channel
                )
                await channel.receive()
            },
            onFinish: { [weak self] in
                self?._actor.notifyEphemeralPeerNegotiated()
            }
        )
    }

    // MARK: - Lifecycle

    func startTunnel(options: StartOptions) async {
        startObservingActorState()
        actor.start(options: options)
    }

    func stopTunnel() async {
        actor.stop()
        await actor.waitUntilDisconnected()
        stopObservingActorState()
    }

    func sleep() async {
        actor.onSleep()
    }

    func wake() {
        actor.onWake()
    }
}

// MARK: - State observer

extension WireGuardGoTunnelImplementation {
    private func startObservingActorState() {
        stopObservingActorState()

        stateObserverTask = Task {
            let stateStream = await self._actor.observedStates
            var lastConnectionAttempt: UInt = 0

            for await newState in stateStream {
                if case .connected = newState {
                    // Toggle reasserting to invalidate old sockets when a new relay connection is up.
                    self.provider?.reasserting = true
                    self.provider?.reasserting = false
                }

                switch newState {
                case let .reconnecting(observedConnectionState), let .connecting(observedConnectionState):
                    let connectionAttempt = observedConnectionState.connectionAttemptCount

                    // Start device check every second failure attempt to connect.
                    if lastConnectionAttempt != connectionAttempt, connectionAttempt > 0,
                        connectionAttempt.isMultiple(of: 2)
                    {
                        onDeviceCheck?()
                    }

                    // Cache last connection attempt to filter out repeating calls.
                    lastConnectionAttempt = connectionAttempt

                case let .negotiatingEphemeralPeer(observedConnectionState, privateKey):
                    await ephemeralPeerExchangingPipeline.startNegotiation(
                        observedConnectionState,
                        privateKey: privateKey
                    )
                case .disconnected:
                    stopDefaultPathObserver()
                case .initial, .connected, .disconnecting, .error:
                    break
                }
            }
        }
    }

    private func stopObservingActorState() {
        stateObserverTask?.cancel()
        stateObserverTask = nil
    }
}

// MARK: - Network path monitor observing

extension WireGuardGoTunnelImplementation {
    private func startDefaultPathObserver() {
        logger.trace("Start default path observer.")

        defaultPathObserver.start { [weak self] networkPath in
            self?._actor.updateNetworkReachability(networkPathStatus: networkPath)
        }
    }

    private func stopDefaultPathObserver() {
        logger.trace("Stop default path observer.")

        defaultPathObserver.stop()
    }
}

// MARK: - EphemeralPeerReceiving

extension WireGuardGoTunnelImplementation: EphemeralPeerReceiving {
    func receivePostQuantumKey(
        _ key: WireGuard.PreSharedKey,
        ephemeralKey: WireGuard.PrivateKey,
        daitaParameters: MullvadTypes.DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchangingPipeline.receivePostQuantumKey(
            key,
            ephemeralKey: ephemeralKey,
            daitaParameters: daitaParameters
        )
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: WireGuard.PrivateKey,
        daitaParameters: MullvadTypes.DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchangingPipeline.receiveEphemeralPeerPrivateKey(
            ephemeralPeerPrivateKey,
            daitaParameters: daitaParameters
        )
    }

    func ephemeralPeerExchangeFailed() {
        // Do not try reconnecting to the `.current` relay, else the actor's `State` equality check will fail
        // and it will not try to reconnect
        actor.reconnect(to: .random, reconnectReason: .connectionLoss)
    }
}
