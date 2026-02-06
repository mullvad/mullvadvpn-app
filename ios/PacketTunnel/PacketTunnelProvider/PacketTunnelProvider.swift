//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
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
    private var appStoreMetaDataService: AppStoreMetaDataService!
    private let tunnelSettingsUpdater: SettingsUpdater
    private let defaultPathObserver: PacketTunnelPathObserver
    private var migrationManager: MigrationManager
    let migrationFailureIterator = REST.RetryStrategy.failedMigrationRecovery.makeDelayIterator()

    private let tunnelSettingsListener = TunnelSettingsListener()
    private lazy var ephemeralPeerReceiver = {
        EphemeralPeerReceiver(tunnelProvider: adapter, keyReceiver: self)
    }()

    var apiContext: MullvadApiContext!
    var accessMethodReceiver: MullvadAccessMethodReceiver!
    private var shadowsocksCacheCleaner: ShadowsocksCacheCleaner!

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

        defaultPathObserver = PacketTunnelPathObserver(eventQueue: internalQueue)

        super.init()

        performSettingsMigration()

        let settingsReader = TunnelSettingsManager(settingsReader: SettingsReader()) { [weak self] settings in
            guard let self = self else { return }
            tunnelSettingsListener.onNewSettings?(settings.tunnelSettings)
        }

        let tunnelSettings = (try? settingsReader.read().tunnelSettings) ?? LatestTunnelSettings()
        let accessMethodRepository = AccessMethodRepository()

        setUpApiContextAndAccessMethodReceiver(
            appContainerURL: containerURL,
            ipOverrideWrapper: ipOverrideWrapper,
            addressCache: addressCache,
            accessMethodRepository: accessMethodRepository,
            tunnelSettings: tunnelSettings
        )

        setUpAccessMethodReceiver(
            accessMethodRepository: accessMethodRepository
        )

        let apiTransportProvider = APITransportProvider(
            requestFactory: MullvadApiRequestFactory(
                apiContext: apiContext,
                encoder: REST.Coding.makeJSONEncoder()
            )
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
            apiTransportProvider: apiTransportProvider
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
            defaultPathObserver: defaultPathObserver,
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: relaySelector,
            settingsReader: settingsReader,
            protocolObfuscator: ProtocolObfuscator<TunnelObfuscator>()
        )

        // Since PacketTunnelActor depends on the path observer, start observing after actor has been initalized.
        startDefaultPathObserver()

        let apiRequestProxy = APIRequestProxy(
            dispatchQueue: internalQueue,
            transportProvider: apiTransportProvider
        )
        appMessageHandler = AppMessageHandler(
            packetTunnelActor: actor,
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
            },
            onFinish: { [unowned self] in
                actor.notifyEphemeralPeerNegotiated()
            }
        )

        appStoreMetaDataService = AppStoreMetaDataService(
            tunnelSettings: tunnelSettings,
            urlSession: URLSession.shared,
            appPreferences: AppPreferences(),
            mainAppBundleIdentifier: ApplicationTarget.mainApp.bundleIdentifier
        )
        #if DEBUG
            appStoreMetaDataService.scheduleTimer()
        #endif
    }

    override func startTunnel(
        options: [String: NSObject]? = nil,
        completionHandler: @escaping @Sendable ((any Error)?) -> Void
    ) {
        let startOptions = parseStartOptions(options ?? [:])

        startObservingActorState()

        // Run device check during tunnel startup.
        // This check is allowed to push new key to server if there are some issues with it.
        startDeviceCheck(rotateKeyOnMismatch: true)

        setTunnelNetworkSettings(
            initialTunnelNetworkSettings(),
            completionHandler: { error in
                if let error {
                    self.providerLogger
                        .error(
                            "Failed to configure tunnel with initial config: \(error)"
                        )
                } else {
                    self.providerLogger.debug("Starting actor after initial configuration is applied")
                    self.actor.start(options: startOptions)
                }
                self.internalQueue.async {
                    completionHandler(error)
                }
            })
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        providerLogger.debug("stopTunnel: \(ProviderStopReasonWrapper(reason: reason))")

        actor.stop()
        await actor.waitUntilDisconnected()

        stopObservingActorState()
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

    private func setUpApiContextAndAccessMethodReceiver(
        appContainerURL: URL,
        ipOverrideWrapper: IPOverrideWrapper,
        addressCache: REST.AddressCache,
        accessMethodRepository: AccessMethodRepository,
        tunnelSettings: LatestTunnelSettings
    ) {
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: appContainerURL)

        let shadowsocksRelaySelector = ShadowsocksRelaySelector(
            relayCache: ipOverrideWrapper
        )

        let shadowsocksLoader = ShadowsocksLoader(
            cache: shadowsocksCache,
            relaySelector: shadowsocksRelaySelector,
            tunnelSettings: tunnelSettings,
            settingsUpdater: tunnelSettingsUpdater
        )

        shadowsocksCacheCleaner = ShadowsocksCacheCleaner(cache: shadowsocksCache)

        let opaqueAccessMethodSettingsWrapper = initAccessMethodSettingsWrapper(
            methods: accessMethodRepository.fetchAll()
        )

        // swift-format-ignore: NeverUseForceTry
        apiContext = try! MullvadApiContext(
            host: REST.defaultAPIHostname,
            address: REST.defaultAPIEndpoint.description,
            domain: REST.encryptedDNSHostname,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: opaqueAccessMethodSettingsWrapper,
            addressCacheProvider: addressCache,
            accessMethodChangeListeners: [accessMethodRepository, shadowsocksCacheCleaner]
        )
    }

    private func setUpAccessMethodReceiver(
        accessMethodRepository: AccessMethodRepository
    ) {
        accessMethodReceiver = MullvadAccessMethodReceiver(
            apiContext: apiContext,
            accessMethodsDataSource: accessMethodRepository.accessMethodsPublisher,
            requestDataSource: accessMethodRepository.requestAccessMethodPublisher
        )
    }

    private func initialTunnelNetworkSettings() -> NETunnelNetworkSettings {
        let settings = NEPacketTunnelNetworkSettings(
            tunnelRemoteAddress: "\(IPv4Address.loopback)"
        )

        // IPv4 settings
        let ipv4Settings = NEIPv4Settings(
            addresses: [LocalNetworkIPs.gatewayAddressIpV4.rawValue],
            subnetMasks: ["255.255.255.255"]
        )
        ipv4Settings.includedRoutes = [NEIPv4Route.default()]
        settings.ipv4Settings = ipv4Settings

        // IPv6 settings
        let ipv6Settings = NEIPv6Settings(
            addresses: [LocalNetworkIPs.gatewayAddressIpV6.rawValue],
            networkPrefixLengths: [128]
        )
        ipv6Settings.includedRoutes = [NEIPv6Route.default()]
        settings.ipv6Settings = ipv6Settings

        return settings
    }
}

extension PacketTunnelProvider {
    private static func configureLogging() {
        let loggerBuilder = LoggerBuilder.shared
        let header = "PacketTunnel version \(Bundle.main.productVersion)"

        loggerBuilder.addFileOutput(
            fileURL: ApplicationConfiguration.newLogFileURL(
                for: .packetTunnel,
                in: ApplicationConfiguration.containerURL
            ),
            header: header
        )
        #if DEBUG
            loggerBuilder.addOSLogOutput(subsystem: ApplicationTarget.packetTunnel.bundleIdentifier)
        #endif
        loggerBuilder.install()

        // Initialize Rust logging to forward to Swift Logger
        RustLogging.initialize()
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

// MARK: - Network path monitor observing

extension PacketTunnelProvider {

    private func startDefaultPathObserver() {
        providerLogger.trace("Start default path observer.")

        defaultPathObserver.start { [weak self] networkPath in
            self?.actor.updateNetworkReachability(networkPathStatus: networkPath)
        }
    }

    private func stopDefaultPathObserver() {
        providerLogger.trace("Stop default path observer.")

        defaultPathObserver.stop()
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
                if case .connected = newState {
                    // Instead of setting the `reasserting` flag to true when we lose connectivity, we can wait until we restore connectivity. This will decrease the amount of path updates that are issued. There's also no need to signal to the system that anything is broken - instead we just want to invalidate old sockets when a new relay connection is up - the only way to do that is to toggle this flag.
                    self.reasserting = true
                    self.reasserting = false
                }

                switch newState {
                case let .reconnecting(observedConnectionState), let .connecting(observedConnectionState):
                    let connectionAttempt = observedConnectionState.connectionAttemptCount

                    // Start device check every second failure attempt to connect.
                    if lastConnectionAttempt != connectionAttempt, connectionAttempt > 0,
                        connectionAttempt.isMultiple(of: 2)
                    {
                        startDeviceCheck()
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
                providerLogger.error("\(error.description) Forcing a log out")
                actor.setErrorState(reason: .deviceLoggedOut)
            default:
                providerLogger
                    .error(
                        "Device check encountered a network error: \(error.description)"
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
