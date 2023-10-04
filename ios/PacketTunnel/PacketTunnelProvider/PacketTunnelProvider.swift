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
import MullvadTransport
import MullvadTypes
import NetworkExtension
import PacketTunnelCore
import RelayCache

class PacketTunnelProvider: NEPacketTunnelProvider {
    private let internalQueue = DispatchQueue(label: "PacketTunnel-internalQueue")
    private let providerLogger: Logger
    private let constraintsUpdater = RelayConstraintsUpdater()

    private var actor: PacketTunnelActor!
    private var appMessageHandler: AppMessageHandler!
    private var stateObserverTask: AnyTask?
    private var deviceChecker: DeviceChecker!
    private var isLoggedSameIP = false

    override init() {
        Self.configureLogging()

        providerLogger = Logger(label: "PacketTunnelProvider")

        let containerURL = ApplicationConfiguration.containerURL
        let addressCache = REST.AddressCache(canWriteToCache: false, cacheDirectory: containerURL)
        addressCache.loadFromFile()

        let relayCache = RelayCache(cacheDirectory: containerURL)

        let urlSession = REST.makeURLSession()
        let urlSessionTransport = URLSessionTransport(urlSession: urlSession)
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: containerURL)

        // This init cannot fail as long as the security group identifier is valid
        let sharedUserDefaults = UserDefaults(suiteName: ApplicationConfiguration.securityGroupIdentifier)!
        let transportStrategy = TransportStrategy(sharedUserDefaults)

        let transportProvider = TransportProvider(
            urlSessionTransport: urlSessionTransport,
            relayCache: relayCache,
            addressCache: addressCache,
            shadowsocksCache: shadowsocksCache,
            transportStrategy: transportStrategy,
            constraintsUpdater: constraintsUpdater
        )

        super.init()

        let adapter = WgAdapter(packetTunnelProvider: self)

        let tunnelMonitor = TunnelMonitor(
            eventQueue: internalQueue,
            pinger: Pinger(replyQueue: internalQueue),
            tunnelDeviceInfo: adapter,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self),
            timings: TunnelMonitorTimings()
        )

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            addressCache: addressCache
        )
        let accountsProxy = proxyFactory.createAccountsProxy()
        let devicesProxy = proxyFactory.createDevicesProxy()

        deviceChecker = DeviceChecker(accountsProxy: accountsProxy, devicesProxy: devicesProxy)

        actor = PacketTunnelActor(
            timings: PacketTunnelActorTimings(),
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self),
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: RelaySelectorWrapper(relayCache: relayCache),
            settingsReader: SettingsReader()
        )

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

        await actor.waitUntilConnected()
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
}

extension PacketTunnelProvider {
    override func setTunnelNetworkSettings(
        _ tunnelNetworkSettings: NETunnelNetworkSettings?,
        completionHandler: ((Error?) -> Void)? = nil
    ) {
        if let networkSettings = tunnelNetworkSettings as? NEPacketTunnelNetworkSettings {
            let ipv4Addresses = networkSettings.ipv4Settings?.addresses.compactMap { IPv4Address($0) } ?? []
            let ipv6Addresses = networkSettings.ipv6Settings?.addresses.compactMap { IPv6Address($0) } ?? []
            let allIPAddresses: [IPAddress] = ipv4Addresses + ipv6Addresses

            if !allIPAddresses.isEmpty, !isLoggedSameIP {
                isLoggedSameIP = true
                logIfDeviceHasSameIP(than: allIPAddresses)
            }
        }

        super.setTunnelNetworkSettings(tunnelNetworkSettings, completionHandler: completionHandler)
    }

    private func logIfDeviceHasSameIP(than addresses: [IPAddress]) {
        let hasIPv4SameAddress = addresses.compactMap { $0 as? IPv4Address }
            .contains { $0 == ApplicationConfiguration.sameIPv4 }
        let hasIPv6SameAddress = addresses.compactMap { $0 as? IPv6Address }
            .contains { $0 == ApplicationConfiguration.sameIPv6 }

        let isUsingSameIP = (hasIPv4SameAddress || hasIPv6SameAddress) ? "" : "NOT "
        providerLogger.debug("Same IP is \(isUsingSameIP)being used")
    }
}

extension PacketTunnelProvider {
    private static func configureLogging() {
        var loggerBuilder = LoggerBuilder()
        let pid = ProcessInfo.processInfo.processIdentifier
        loggerBuilder.metadata["pid"] = .string("\(pid)")
        loggerBuilder.addFileOutput(fileURL: ApplicationConfiguration.logFileURL(for: .packetTunnel))
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
            let stateStream = await self.actor.states
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
