//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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

class PacketTunnelProvider: NEPacketTunnelProvider, @unchecked Sendable {
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
    private var encryptedDNSTransport: EncryptedDNSTransport!
    private var migrationManager: MigrationManager!
    let migrationFailureIterator = REST.RetryStrategy.failedMigrationRecovery.makeDelayIterator()

    private let tunnelSettingsListener = TunnelSettingsListener()
    private lazy var ephemeralPeerReceiver = {
        EphemeralPeerReceiver(tunnelProvider: adapter, keyReceiver: self)
    }()

    // swiftlint:disable:next function_body_length
    override init() {
        Self.configureLogging()
        providerLogger = Logger(label: "PacketTunnelProvider")
        providerLogger.info("Starting new packet tunnel")

        let containerURL = ApplicationConfiguration.containerURL
        let addressCache = REST.AddressCache(canWriteToCache: false, cacheDirectory: containerURL)
        addressCache.loadFromFile()

        let ipOverrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(cacheDirectory: containerURL),
            ipOverrideRepository: IPOverrideRepository()
        )
        tunnelSettingsUpdater = SettingsUpdater(listener: tunnelSettingsListener)
        migrationManager = MigrationManager(cacheDirectory: containerURL)

        super.init()

        performSettingsMigration()

        let transportProvider = setUpTransportProvider(
            appContainerURL: containerURL,
            ipOverrideWrapper: ipOverrideWrapper,
            addressCache: addressCache
        )

        let apiTransportProvider = APITransportProvider(
            requestFactory: MullvadApiRequestFactory(apiContext: REST.apiContext)
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
            apiTransportProvider: apiTransportProvider,
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
            defaultPathObserver: PacketTunnelPathObserver(eventQueue: internalQueue),
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: relaySelector,
            settingsReader: TunnelSettingsManager(settingsReader: SettingsReader()) { [weak self] settings in
                guard let self = self else { return }
                tunnelSettingsListener.onNewSettings?(settings.tunnelSettings)
            },
            protocolObfuscator: ProtocolObfuscator<TunnelObfuscator>()
        )

        let urlRequestProxy = URLRequestProxy(
            dispatchQueue: internalQueue,
            transportProvider: transportProvider
        )
        let apiRequestProxy = APIRequestProxy(
            dispatchQueue: internalQueue,
            transportProvider: apiTransportProvider
        )
        appMessageHandler = AppMessageHandler(
            packetTunnelActor: actor,
            urlRequestProxy: urlRequestProxy,
            apiRequestProxy: apiRequestProxy
        )

        ephemeralPeerExchangingPipeline = EphemeralPeerExchangingPipeline(
            EphemeralPeerExchangeActor(
                packetTunnel: ephemeralPeerReceiver,
                onFailure: self.ephemeralPeerExchangeFailed,
                iteratorProvider: { REST.RetryStrategy.postQuantumKeyExchange.makeDelayIterator() }
            ),
            onUpdateConfiguration: { [unowned self] configuration in
                let channel = OneshotChannel()
                actor.changeEphemeralPeerNegotiationState(
                    configuration: configuration,
                    reconfigurationSemaphore: channel
                )
                await channel.receive()
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

    private func performSettingsMigration() {
        nonisolated(unsafe) var hasNotMigrated = true
        repeat {
            migrationManager.migrateSettings(
                store: SettingsManager.store,
                migrationCompleted: { [unowned self] migrationResult in
                    switch migrationResult {
                    case .success:
                        providerLogger.debug("Successful migration from PacketTunnel")
                        hasNotMigrated = false
                    case .nothing:
                        hasNotMigrated = false
                        providerLogger.debug("Attempted migration from PacketTunnel, but found nothing to do")
                    case let .failure(error):
                        providerLogger
                            .error(
                                "Failed migration from PacketTunnel: \(error)"
                            )
                    }
                }
            )
            if hasNotMigrated {
                // `next` returns an Optional value, but this iterator is guaranteed to always have a next value
                guard let delay = migrationFailureIterator.next() else { continue }

                providerLogger.error("Retrying migration in \(delay.timeInterval) seconds")
                // Block the launch of the Packet Tunnel for as long as the settings migration fail.
                // The process watchdog introduced by iOS 17 will kill this process after 60 seconds.
                Thread.sleep(forTimeInterval: delay.timeInterval)
            }
        } while hasNotMigrated
    }

    private func setUpTransportProvider(
        appContainerURL: URL,
        ipOverrideWrapper: IPOverrideWrapper,
        addressCache: REST.AddressCache
    ) -> TransportProvider {
        let urlSession = REST.makeURLSession(addressCache: addressCache)
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

        encryptedDNSTransport = EncryptedDNSTransport(urlSession: urlSession)
        return TransportProvider(
            urlSessionTransport: urlSessionTransport,
            addressCache: addressCache,
            transportStrategy: transportStrategy,
            encryptedDNSTransport: encryptedDNSTransport
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
                    await ephemeralPeerExchangingPipeline.startNegotiation(
                        observedConnectionState,
                        privateKey: privateKey
                    )
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
    func receivePostQuantumKey(
        _ key: PreSharedKey,
        ephemeralKey: PrivateKey,
        daitaParameters: MullvadTypes.DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchangingPipeline.receivePostQuantumKey(
            key,
            ephemeralKey: ephemeralKey,
            daitaParameters: daitaParameters
        )
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: PrivateKey,
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
