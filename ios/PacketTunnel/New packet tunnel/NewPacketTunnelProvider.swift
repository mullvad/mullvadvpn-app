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

    /// Request proxy used to perform URLRequests bypassing VPN.
    private let urlRequestProxy: URLRequestProxy

    private var adapter: WgAdapter!
    private var tunnelMonitor: TunnelMonitor!
    private var actor: PacketTunnelActor!

    private var stateObserverTask: AnyTask?

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

        adapter = WgAdapter(packetTunnelProvider: self)

        tunnelMonitor = TunnelMonitor(
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

        actor = PacketTunnelActor(
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: PacketTunnelPathObserver(packetTunnelProvider: self),
            relaySelector: RelaySelectorWrapper(relayCache: relayCache),
            settingsReader: SettingsReader(),
            deviceChecker: DeviceChecker(accountsProxy: accountsProxy, devicesProxy: devicesProxy)
        )
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        let startOptions = parseStartOptions(options ?? [:])

        startObservingActorState()

        try await actor.start(options: startOptions)
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        stopObservingActorState()

        await actor.stop()
    }

    override func handleAppMessage(_ messageData: Data) async -> Data? {
        guard let message = decodeMessage(messageData) else { return nil }

        providerLogger.trace("Received app message: \(message)")

        switch message {
        case let .sendURLRequest(request):
            return await encodeReply(urlRequestProxy.sendRequest(request))

        case let .cancelURLRequest(id):
            urlRequestProxy.cancelRequest(identifier: id)

        case .getTunnelStatus:
            return await encodeReply(actor.state.packetTunnelStatus)

        case .privateKeyRotation:
            await actor.notifyKeyRotated()

        case let .reconnectTunnel(selectorResult):
            try? await actor.reconnect(to: selectorResult.map { .preSelected($0) } ?? .random)
        }

        return nil
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

    private func decodeMessage(_ data: Data) -> TunnelProviderMessage? {
        do {
            return try TunnelProviderMessage(messageData: data)
        } catch {
            providerLogger.error(error: error, message: "Failed to decode the app message.")
            return nil
        }
    }

    private func encodeReply<T: Codable>(_ reply: T) -> Data? {
        do {
            return try TunnelProviderReply(reply).encode()
        } catch {
            providerLogger.error(error: error, message: "Failed to decode the app message.")
            return nil
        }
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
            }
        }
    }

    private func stopObservingActorState() {
        stateObserverTask?.cancel()
        stateObserverTask = nil
    }
}
