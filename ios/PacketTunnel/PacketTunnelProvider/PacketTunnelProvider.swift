//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 31/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadPostQuantum
import MullvadREST
import MullvadSettings
import MullvadTypes
import NetworkExtension
import PacketTunnelCore
import TunnelObfuscation
import WireGuardKitTypes

class PacketTunnelProvider: NEPacketTunnelProvider {
    private let internalQueue = DispatchQueue(label: "PacketTunnel-internalQueue")
    private let providerLogger: Logger
    private let constraintsUpdater = RelayConstraintsUpdater()

    private var actor: PacketTunnelActor!
    private var postQuantumActor: PostQuantumKeyExchangeActor!
    private var appMessageHandler: AppMessageHandler!
    private var stateObserverTask: AnyTask?
    private var deviceChecker: DeviceChecker!
    private var adapter: WgAdapter!
    private var relaySelector: RelaySelectorWrapper!

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

        super.init()

        let transportProvider = setUpTransportProvider(
            appContainerURL: containerURL,
            ipOverrideWrapper: ipOverrideWrapper,
            addressCache: addressCache
        )

        adapter = WgAdapter(packetTunnelProvider: self)

        let tunnelMonitor = TunnelMonitor(
            eventQueue: internalQueue,
            pinger: Pinger(replyQueue: internalQueue),
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
        relaySelector = RelaySelectorWrapper(relayCache: ipOverrideWrapper)

        actor = PacketTunnelActor(
            timings: PacketTunnelActorTimings(),
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self, eventQueue: internalQueue),
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: relaySelector,
            settingsReader: SettingsReader(),
            protocolObfuscator: ProtocolObfuscator<UDPOverTCPObfuscator>()
        )

        postQuantumActor = PostQuantumKeyExchangeActor(packetTunnel: self)

        let urlRequestProxy = URLRequestProxy(dispatchQueue: internalQueue, transportProvider: transportProvider)

        appMessageHandler = AppMessageHandler(packetTunnelActor: actor, urlRequestProxy: urlRequestProxy)
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
            case .negotiatingPostQuantumKey:
                // When negotiating post quantum keys, allow the connection to go through immediately.
                // Otherwise, the in-tunnel TCP connection will never become ready as the OS doesn't let
                // any traffic through until this function returns, which would prevent negotiating keys
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

        let transportStrategy = TransportStrategy(
            datasource: AccessMethodRepository(),
            shadowsocksLoader: ShadowsocksLoader(
                shadowsocksCache: shadowsocksCache,
                relayCache: ipOverrideWrapper,
                constraintsUpdater: constraintsUpdater
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
        loggerBuilder.addFileOutput(fileURL: ApplicationConfiguration.newLogFileURL(for: .packetTunnel))
        #if DEBUG
        loggerBuilder.addOSLogOutput(subsystem: ApplicationTarget.packetTunnel.bundleIdentifier)
        #endif
        loggerBuilder.install()
    }

    private func parseStartOptions(_ options: [String: NSObject]) -> StartOptions {
        let tunnelOptions = PacketTunnelOptions(rawOptions: options)
        var parsedOptions = StartOptions(launchSource: tunnelOptions.isOnDemand() ? .onDemand : .app)

        do {
            if let selectedRelay = try tunnelOptions.getSelectedRelay() {
                parsedOptions.launchSource = .app
                parsedOptions.selectedRelay = selectedRelay
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
                // Pass relay constraints retrieved during the last read from setting into transport provider.
                if let relayConstraints = newState.relayConstraints {
                    constraintsUpdater.onNewConstraints?(relayConstraints)
                }

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
                case let .reconnecting(connState), let .connecting(connState):
                    let connectionAttempt = connState.connectionAttemptCount

                    // Start device check every second failure attempt to connect.
                    if lastConnectionAttempt != connectionAttempt, connectionAttempt > 0,
                       connectionAttempt.isMultiple(of: 2) {
                        startDeviceCheck()
                    }

                    // Cache last connection attempt to filter out repeating calls.
                    lastConnectionAttempt = connectionAttempt

                case let .negotiatingPostQuantumKey(_, privateKey):
                    postQuantumActor.startNegotiation(with: privateKey)

                case .initial, .connected, .disconnecting, .disconnected, .error:
                    break
                }
            }
        }
    }

    func createTCPConnectionForPQPSK(_ gatewayAddress: String) -> NWTCPConnection {
        let gatewayEndpoint = NWHostEndpoint(hostname: gatewayAddress, port: "1337")
        return createTCPConnectionThroughTunnel(
            to: gatewayEndpoint,
            enableTLS: false,
            tlsParameters: nil,
            delegate: nil
        )
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
            do {
                try await startDeviceCheckInner(rotateKeyOnMismatch: rotateKeyOnMismatch)
            } catch {
                providerLogger.error(error: error, message: "Failed to perform device check.")
            }
        }
    }

    private func startDeviceCheckInner(rotateKeyOnMismatch: Bool) async throws {
        let result = try await deviceChecker.start(rotateKeyOnMismatch: rotateKeyOnMismatch)

        if let blockedStateReason = result.blockedStateReason {
            actor.setErrorState(reason: blockedStateReason)
        }

        switch result.keyRotationStatus {
        case let .attempted(date), let .succeeded(date):
            actor.notifyKeyRotation(date: date)
        case .noAction:
            break
        }
    }
}

extension PacketTunnelProvider: PostQuantumKeyReceiving {
    func receivePostQuantumKey(_ key: PreSharedKey, ephemeralKey: PrivateKey) {
        actor.replacePreSharedKey(key, ephemeralKey: ephemeralKey)
        postQuantumActor.acknowledgeNegotiationConcluded()
    }

    func keyExchangeFailed() {
        postQuantumActor.acknowledgeNegotiationConcluded()
        actor.reconnect(to: .current)
    }
}
