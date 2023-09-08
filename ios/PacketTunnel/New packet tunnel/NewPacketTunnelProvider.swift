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

    // TODO: pass relay constraints into RelayConstraintsUpdater
    private let constraintsUpdater = RelayConstraintsUpdater()

    /// Request proxy used to perform URLRequests bypassing VPN.
    private let urlRequestProxy: URLRequestProxy

    private var adapter: WgAdapter!
    private var tunnelMonitor: TunnelMonitor!
    private var actor: PacketTunnelActor!

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

        actor = PacketTunnelActor(
            tunnelAdapter: adapter,
            tunnelMonitor: tunnelMonitor,
            relaySelector: RelaySelectorWrapper(relayCache: relayCache),
            settingsReader: SettingsReader()
        )
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        let startOptions = parseStartOptions(options ?? [:])
        try await actor.start(options: startOptions)
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
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
            return nil

        case .getTunnelStatus:
            return await encodeReply(actor.state.packetTunnelStatus)

        case .privateKeyRotation:
            // TODO: tell actor that key rotation has happened
            return nil

        case let .reconnectTunnel(selectorResult):
            return nil
        }
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
}

extension State {
    var packetTunnelStatus: PacketTunnelStatus {
        var status = PacketTunnelStatus()

        switch self {
        case let .connecting(connState),
             let .connected(connState),
             let .reconnecting(connState),
             let .disconnecting(connState):
            switch connState.networkReachability {
            case .reachable:
                status.isNetworkReachable = true
            case .unreachable:
                status.isNetworkReachable = false
            case .undetermined:
                // TODO: fix me
                status.isNetworkReachable = true
            }

            status.numberOfFailedAttempts = connState.connectionAttemptCount
            status.tunnelRelay = connState.selectedRelay.packetTunnelRelay

        case .disconnected, .initial:
            break

        case let .error(blockedState):
            // Later?
            status.lastErrors = []
        }

        return status
    }
}
