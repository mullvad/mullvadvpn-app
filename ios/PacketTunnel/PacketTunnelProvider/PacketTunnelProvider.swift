//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import NetworkExtension
import PacketTunnelCore
import WireGuardKitTypes

class PacketTunnelProvider: NEPacketTunnelProvider {
    private let internalQueue = DispatchQueue(label: "PacketTunnel-internalQueue")
    private let providerLogger: Logger

    private var actor: PacketTunnelActor!
    private var appMessageHandler: AppMessageHandler!
    private var stateObserverTask: AnyTask?
    private var deviceChecker: DeviceChecker!
    private var adapter: WgAdapter!
    private var relaySelector: RelaySelectorWrapper!
    private var ephemeralPeerExchangingPipeline: EphemeralPeerExchangingPipeline!
    private let tunnelSettingsUpdater: SettingsUpdater!

    private let tunnelSettingsListener = TunnelSettingsListener()
    private lazy var ephemeralPeerReceiver = {
        EphemeralPeerReceiver(tunnelProvider: self)
    }()

    // swiftlint:disable:next function_body_length
    override init() {
        Self.configureLogging()
        providerLogger = Logger(label: "PacketTunnelProvider")

        let containerURL = ApplicationConfiguration.containerURL
        let addressCache = REST.AddressCache(canWriteToCache: false, cacheDirectory: containerURL)
        addressCache.loadFromFile()

        let ipOverrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(cacheDirectory: containerURL),
            ipOverrideRepository: IPOverrideRepository()
        )
        tunnelSettingsUpdater = SettingsUpdater(listener: tunnelSettingsListener)

        super.init()

        let transportProvider = setUpTransportProvider(
            appContainerURL: containerURL,
            ipOverrideWrapper: ipOverrideWrapper,
            addressCache: addressCache
        )

        adapter = WgAdapter(packetTunnelProvider: self)

        let pinger = TunnelPinger(pingProvider: adapter.icmpPingProvider, replyQueue: internalQueue)

        let tunnelMonitor = TunnelMonitor(
            eventQueue: internalQueue,
            pinger: pinger,
            tunnelDeviceInfo: adapter,
            timings: TunnelMonitorTimings()
        )

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            addressCache: addressCache
        )
        let accountsProxy = proxyFactory.createAccountsProxy()
        let devicesProxy = proxyFactory.createDevicesProxy()

        deviceChecker = DeviceChecker(accountsProxy: accountsProxy, devicesProxy: devicesProxy)
        relaySelector = RelaySelectorWrapper(
            relayCache: ipOverrideWrapper
        )

        actor = PacketTunnelActor(
            timings: PacketTunnelActorTimings(),
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self, eventQueue: internalQueue),
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: relaySelector,
            settingsReader: TunnelSettingsManager(settingsReader: SettingsReader()) { [weak self] settings in
                guard let self = self else { return }
                tunnelSettingsListener.onNewSettings?(settings.tunnelSettings)
            },
            protocolObfuscator: ProtocolObfuscator<UDPOverTCPObfuscator>()
        )

        let urlRequestProxy = URLRequestProxy(dispatchQueue: internalQueue, transportProvider: transportProvider)
        appMessageHandler = AppMessageHandler(packetTunnelActor: actor, urlRequestProxy: urlRequestProxy)

        ephemeralPeerExchangingPipeline = EphemeralPeerExchangingPipeline(
            EphemeralPeerExchangeActor(
                packetTunnel: ephemeralPeerReceiver,
                onFailure: self.ephemeralPeerExchangeFailed,
                iteratorProvider: { REST.RetryStrategy.postQuantumKeyExchange.makeDelayIterator() }
            ),
            onUpdateConfiguration: { [unowned self] configuration in
                actor.changeEphemeralPeerNegotiationState(configuration: configuration)
            }, onFinish: { [unowned self] in
                actor.notifyEphemeralPeerNegotiated()
            }
        )
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        let startOptions = parseStartOptions(options ?? [:])

        startObservingActorState()

        // Run device check during tunnel startup.
        // This check is allowed to push new key to server if there are some issues with it.
        startDeviceCheck(rotateKeyOnMismatch: true)

        actor.start(options: startOptions)

        for await state in await actor.observedStates {
            switch state {
            case .connected, .disconnected, .error:
                return
            case let .connecting(connectionState):
                // Give the tunnel a few tries to connect, otherwise return immediately. This will enable VPN in
                // device settings, but the app will still report the true state via ObservedState over IPC.
                // In essence, this prevents the 60s tunnel timeout to trigger.
                if connectionState.connectionAttemptCount > 1 {
                    return
                }
            case .negotiatingEphemeralPeer:
                // When negotiating ephemeral peers, allow the connection to go through immediately.
                // Otherwise, the in-tunnel TCP connection will never become ready as the OS doesn't let
                // any traffic through until this function returns, which would prevent negotiating ephemeral peers
                // from an unconnected state.
                return
            default:
                break
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        providerLogger.debug("stopTunnel: \(reason)")

        stopObservingActorState()

        actor.stop()

        await actor.waitUntilDisconnected()
    }

    override func handleAppMessage(_ messageData: Data) async -> Data? {
        return await appMessageHandler.handleAppMessage(messageData)
    }

    override func sleep() async {
        actor.onSleep()
    }

    override func wake() {
        actor.onWake()
    }

    private func setUpTransportProvider(
        appContainerURL: URL,
        ipOverrideWrapper: IPOverrideWrapper,
        addressCache: REST.AddressCache
    ) -> TransportProvider {
        let urlSession = REST.makeURLSession()
        let urlSessionTransport = URLSessionTransport(urlSession: urlSession)
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: appContainerURL)

        let shadowsocksRelaySelector = ShadowsocksRelaySelector(
            relayCache: ipOverrideWrapper
        )

        let transportStrategy = TransportStrategy(
            datasource: AccessMethodRepository(),
            shadowsocksLoader: ShadowsocksLoader(
                cache: shadowsocksCache,
                relaySelector: shadowsocksRelaySelector,
                settingsUpdater: tunnelSettingsUpdater
            )
        )

        return TransportProvider(
            urlSessionTransport: urlSessionTransport,
            addressCache: addressCache,
            transportStrategy: transportStrategy
        )
    }
}

extension PacketTunnelProvider {
    private static func configureLogging() {
        var loggerBuilder = LoggerBuilder(header: "PacketTunnel version \(Bundle.main.productVersion)")
        let pid = ProcessInfo.processInfo.processIdentifier
        loggerBuilder.metadata["pid"] = .string("\(pid)")
        loggerBuilder.addFileOutput(
            fileURL: ApplicationConfiguration.newLogFileURL(
                for: .packetTunnel,
                in: ApplicationConfiguration.containerURL
            )
        )
        #if DEBUG
        loggerBuilder.addOSLogOutput(subsystem: ApplicationTarget.packetTunnel.bundleIdentifier)
        #endif
        loggerBuilder.install()
    }

    private func parseStartOptions(_ options: [String: NSObject]) -> StartOptions {
        let tunnelOptions = PacketTunnelOptions(rawOptions: options)
        var parsedOptions = StartOptions(launchSource: tunnelOptions.isOnDemand() ? .onDemand : .app)

        do {
            if let selectedRelays = try tunnelOptions.getSelectedRelays() {
                parsedOptions.launchSource = .app
                parsedOptions.selectedRelays = selectedRelays
            } else if !tunnelOptions.isOnDemand() {
                parsedOptions.launchSource = .system
            }
        } catch {
            providerLogger.error(error: error, message: "Failed to decode relay selector result passed from the app.")
        }

        return parsedOptions
    }
}

// MARK: - State observer

extension PacketTunnelProvider {
    private func startObservingActorState() {
        stopObservingActorState()

        stateObserverTask = Task {
            let stateStream = await self.actor.observedStates
            var lastConnectionAttempt: UInt = 0

            for await newState in stateStream {
                // Tell packet tunnel when reconnection begins.
                // Packet tunnel moves to `NEVPNStatus.reasserting` state once `reasserting` flag is set to `true`.
                if case .reconnecting = newState, !self.reasserting {
                    self.reasserting = true
                }

                // Tell packet tunnel when reconnection ends.
                // Packet tunnel moves to `NEVPNStatus.connected` state once `reasserting` flag is set to `false`.
                if case .connected = newState, self.reasserting {
                    self.reasserting = false
                }

                switch newState {
                case let .reconnecting(observedConnectionState), let .connecting(observedConnectionState):
                    let connectionAttempt = observedConnectionState.connectionAttemptCount

                    // Start device check every second failure attempt to connect.
                    if lastConnectionAttempt != connectionAttempt, connectionAttempt > 0,
                       connectionAttempt.isMultiple(of: 2) {
                        startDeviceCheck()
                    }

                    // Cache last connection attempt to filter out repeating calls.
                    lastConnectionAttempt = connectionAttempt

                case let .negotiatingEphemeralPeer(observedConnectionState, privateKey):
                    ephemeralPeerExchangingPipeline.startNegotiation(observedConnectionState, privateKey: privateKey)
                case .initial, .connected, .disconnecting, .disconnected, .error:
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

// MARK: - Device check

extension PacketTunnelProvider {
    private func startDeviceCheck(rotateKeyOnMismatch: Bool = false) {
        Task {
            await startDeviceCheckInner(rotateKeyOnMismatch: rotateKeyOnMismatch)
        }
    }

    private func startDeviceCheckInner(rotateKeyOnMismatch: Bool) async {
        let result = await deviceChecker.start(rotateKeyOnMismatch: rotateKeyOnMismatch)

        switch result {
        case let .failure(error):
            switch error {
            case is DeviceCheckError:
                providerLogger.error("\(error.localizedDescription) Forcing a log out")
                actor.setErrorState(reason: .deviceLoggedOut)
            default:
                providerLogger
                    .error(
                        "Device check encountered a network error: \(error.localizedDescription)"
                    )
            }

        case let .success(keyRotationResult):
            if let blockedStateReason = keyRotationResult.blockedStateReason {
                providerLogger.error("Entering blocked state after unsuccessful device check: \(blockedStateReason)")
                actor.setErrorState(reason: blockedStateReason)
                return
            }

            switch keyRotationResult.keyRotationStatus {
            case let .attempted(date), let .succeeded(date):
                actor.notifyKeyRotation(date: date)
            case .noAction:
                break
            }
        }
    }
}

extension PacketTunnelProvider: EphemeralPeerReceiving {
    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        ephemeralPeerExchangingPipeline.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
    }

    public func receiveEphemeralPeerPrivateKey(_ ephemeralPeerPrivateKey: PrivateKey) {
        ephemeralPeerExchangingPipeline.receiveEphemeralPeerPrivateKey(ephemeralPeerPrivateKey)
    }

    func ephemeralPeerExchangeFailed() {
        // Do not try reconnecting to the `.current` relay, else the actor's `State` equality check will fail
        // and it will not try to reconnect
        actor.reconnect(to: .random, reconnectReason: .connectionLoss)
    }
}
