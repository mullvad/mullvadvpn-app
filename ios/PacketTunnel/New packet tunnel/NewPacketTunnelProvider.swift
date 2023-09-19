//
//  NewPacketTunnelProvider.swift
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
import TunnelProviderMessaging

class NewPacketTunnelProvider: NEPacketTunnelProvider {
    private let internalQueue = DispatchQueue(label: "PacketTunnel-internalQueue")
    private let providerLogger: Logger
    private let relayCache: RelayCache
    private let constraintsUpdater = RelayConstraintsUpdater()
    private let urlRequestProxy: URLRequestProxy

    private var actor: PacketTunnelActor!
    private var appMessageHandler: AppMessageHandler!
    private var stateObserverTask: AnyTask?
    private var deviceChecker: DeviceChecker!

    override init() {
        Self.configureLogging()

        providerLogger = Logger(label: "PacketTunnelProvider")

        let containerURL = ApplicationConfiguration.containerURL
        let addressCache = REST.AddressCache(canWriteToCache: false, cacheDirectory: containerURL)
        addressCache.loadFromFile()

        relayCache = RelayCache(cacheDirectory: containerURL)

        let urlSession = REST.makeURLSession()
        let urlSessionTransport = URLSessionTransport(urlSession: urlSession)
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: containerURL)
        let transportProvider = TransportProvider(
            urlSessionTransport: urlSessionTransport,
            relayCache: relayCache,
            addressCache: addressCache,
            shadowsocksCache: shadowsocksCache,
            constraintsUpdater: constraintsUpdater
        )

        urlRequestProxy = URLRequestProxy(dispatchQueue: internalQueue, transportProvider: transportProvider)

        super.init()

        let adapter = WgAdapter(packetTunnelProvider: self)

        let tunnelMonitor = TunnelMonitor(
            eventQueue: internalQueue,
            pinger: Pinger(replyQueue: internalQueue),
            tunnelDeviceInfo: adapter,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self)
        )

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            addressCache: addressCache
        )
        let accountsProxy = proxyFactory.createAccountsProxy()
        let devicesProxy = proxyFactory.createDevicesProxy()

        deviceChecker = DeviceChecker(accountsProxy: accountsProxy, devicesProxy: devicesProxy)

        actor = PacketTunnelActor(
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self),
            blockedStateErrorMapper: BlockedStateErrorMapper(),
            relaySelector: RelaySelectorWrapper(relayCache: relayCache),
            settingsReader: SettingsReader()
        )

        appMessageHandler = AppMessageHandler(packetTunnelActor: actor, urlRequestProxy: urlRequestProxy)
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        let startOptions = parseStartOptions(options ?? [:])

        startObservingActorState()

        // Run device check during tunnel startup.
        // This check is allowed to push new key to server if there are some issues with it.
        startDeviceCheck(rotateKeyOnMismatch: true)

        try await actor.start(options: startOptions)
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        stopObservingActorState()

        await actor.stop()
    }

    override func handleAppMessage(_ messageData: Data) async -> Data? {
        return await appMessageHandler.handleAppMessage(messageData)
    }

    override func sleep() async {
        await actor.onSleep()
    }

    override func wake() {
        actor.onWake()
    }
}

extension NewPacketTunnelProvider {
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
            if let selectorResult = try tunnelOptions.getSelectorResult() {
                parsedOptions.launchSource = .app
                parsedOptions.selectorResult = selectorResult
            } else {
                parsedOptions.launchSource = tunnelOptions.isOnDemand() ? .onDemand : .system
            }
        } catch {
            providerLogger.error(error: error, message: "Failed to decode relay selector result passed from the app.")
        }

        return parsedOptions
    }
}

// MARK: - State observer

extension NewPacketTunnelProvider {
    private func startObservingActorState() {
        stopObservingActorState()

        stateObserverTask = Task {
            let stateStream = await self.actor.states

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
                    // Start device check every second failure attempt to connect.
                    if connState.connectionAttemptCount.isMultiple(of: 2) {
                        startDeviceCheck()
                    }
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

extension NewPacketTunnelProvider {
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
            await actor.setErrorState(with: blockedStateReason)
        }

        switch result.keyRotationStatus {
        case let .attempted(date), let .succeeded(date):
            await actor.notifyKeyRotated(date: date)
        case .noAction:
            break
        }
    }
}
