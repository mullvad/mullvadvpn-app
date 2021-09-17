//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import NetworkExtension
import Logging
import WireGuardKit

class PacketTunnelProvider: NEPacketTunnelProvider {

    /// Tunnel provider logger
    private let providerLogger: Logger

    /// WireGuard adapter logger
    private let tunnelLogger: Logger

    /// Internal queue
    private let dispatchQueue = DispatchQueue(label: "PacketTunnel", qos: .utility)

    /// WireGuard adapter
    private lazy var adapter: WireGuardAdapter = {
        return WireGuardAdapter(with: self, logHandler: { [weak self] (logLevel, message) in
            self?.dispatchQueue.async {
                self?.tunnelLogger.log(level: logLevel.loggerLevel, "\(message)")
            }
        })
    }()

    /// Tunnel connection info
    private var tunnelConnectionInfo: TunnelConnectionInfo? {
        didSet {
            if let tunnelConnectionInfo = tunnelConnectionInfo {
                self.providerLogger.debug("Set tunnel relay to \(tunnelConnectionInfo.hostname)")
            } else {
                self.providerLogger.debug("Unset tunnel relay")
            }
        }
    }

    override init() {
        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!)

        var providerLogger = Logger(label: "PacketTunnelProvider")
        var tunnelLogger = Logger(label: "WireGuard")

        let pid = ProcessInfo.processInfo.processIdentifier
        providerLogger[metadataKey: "pid"] = .stringConvertible(pid)
        tunnelLogger[metadataKey: "pid"] = .stringConvertible(pid)

        self.providerLogger = providerLogger
        self.tunnelLogger = tunnelLogger
    }

    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])
        let appSelectorResult: Result<RelaySelectorResult?, Error> = Result { try tunnelOptions.getSelectorResult() }

        switch appSelectorResult {
        case .success(.some(let selectorResult)):
            providerLogger.debug("Start the tunnel via app, connect to \(selectorResult.tunnelConnectionInfo.hostname)")

        case .success(nil):
            if tunnelOptions.isOnDemand() {
                providerLogger.debug("Start the tunnel via on-demand rule")
            } else {
                providerLogger.debug("Start the tunnel via system")
            }

        case .failure(let error):
            providerLogger.debug("Start the tunnel via app")
            providerLogger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to decode relay selector result passed from the app. Will continue by picking new relay."
            )
        }

        makeConfiguration(appSelectorResult.flattenValue)
            .asPromise()
            .receive(on: dispatchQueue)
            .mapThen { tunnelConfiguration in
                let tunnelConnectionInfo = tunnelConfiguration.selectorResult.tunnelConnectionInfo
                self.tunnelConnectionInfo = tunnelConnectionInfo

                return self.adapter.start(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig)
                    .mapError { error in
                        return PacketTunnelProviderError.startWireguardAdapter(error)
                    }
                    .receive(on: self.dispatchQueue)
            }
            .onSuccess {
                self.providerLogger.debug("Started the tunnel")
            }
            .onFailure { error in
                self.providerLogger.error(chainedError: error, message: "Failed to start the tunnel")
            }
            .observe { completion in
                completionHandler(completion.unwrappedValue?.error)
            }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        providerLogger.debug("Stop the tunnel: \(reason)")

        adapter.stop()
            .receive(on: self.dispatchQueue)
            .mapError { error in
                return PacketTunnelProviderError.stopWireguardAdapter(error)
            }
            .onFailure { error in
                self.providerLogger.error(chainedError: error, message: "Failed to stop the tunnel gracefully")
            }
            .observe { _ in
                self.providerLogger.debug("Stopped the tunnel")
                completionHandler()
            }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        Result { try TunnelIPC.Coding.decodeRequest(messageData) }
            .mapError { PacketTunnelProviderError.ipcHandler($0) }
            .asPromise()
            .onFailure { error in
                self.providerLogger.error(chainedError: error, message: "Failed to decode the app message request")
            }
            .receive(on: dispatchQueue)
            .mapThen { request -> Result<Data?, PacketTunnelProviderError>.Promise in
                self.providerLogger.debug("handleAppMessage: \(request)")

                switch request {
                case .reloadTunnelSettings:
                    self.reloadTunnelSettings().observe { _ in }
                    return .success(nil)

                case .tunnelConnectionInfo:
                    return Result { try TunnelIPC.Coding.encodeResponse(self.tunnelConnectionInfo) }
                        .mapError { PacketTunnelProviderError.ipcHandler($0) }
                        .map { data -> Data? in
                            return .some(data)
                        }
                        .flatMapError { error in
                            self.providerLogger.error(chainedError: error, message: "Failed to encode the app message response for \(request)")
                            return .success(nil)
                        }
                        .asPromise()
                }
            }.observe { completion in
                completionHandler?(completion.unwrappedValue?.value ?? nil)
            }
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        // Add code here to get ready to sleep.
        completionHandler()
    }

    override func wake() {
        // Add code here to wake up.
    }

    // MARK: - Private

    private func makeConfiguration(_ appSelectorResult: RelaySelectorResult? = nil) -> Result<PacketTunnelConfiguration, PacketTunnelProviderError> {
        return protocolConfiguration.passwordReference.map { data in
            return Self.readTunnelSettings(keychainReference: data)
                .flatMap { tunnelSettings in
                    return (appSelectorResult.map { .success($0) } ?? Self.selectRelayEndpoint(relayConstraints: tunnelSettings.relayConstraints))
                        .map { (selectorResult) -> PacketTunnelConfiguration in
                            return PacketTunnelConfiguration(
                                tunnelSettings: tunnelSettings,
                                selectorResult: selectorResult
                            )
                        }
                }
        } ?? .failure(.missingKeychainConfigurationReference)
    }

    private func reloadTunnelSettings() -> Result<(), PacketTunnelProviderError>.Promise {
        providerLogger.debug("Reload tunnel settings")

        return makeConfiguration()
            .asPromise()
            .mapThen { packetTunnelConfig in
                let tunnelConnectionInfo = packetTunnelConfig.selectorResult.tunnelConnectionInfo
                let oldTunnelConnectionInfo = self.tunnelConnectionInfo
                self.tunnelConnectionInfo = tunnelConnectionInfo

                return self.adapter.update(tunnelConfiguration: packetTunnelConfig.wgTunnelConfig)
                    .receive(on: self.dispatchQueue)
                    .mapError { error in
                        return PacketTunnelProviderError.updateWireguardConfiguration(error)
                    }
                    .onSuccess { _ in
                        self.providerLogger.debug("Updated WireGuard configuration")
                    }
                    .onFailure { error in
                        self.tunnelConnectionInfo = oldTunnelConnectionInfo
                        self.providerLogger.error(chainedError: error, message: "Failed to update WireGuard configuration")
                    }
            }
    }

    /// Read tunnel settings from Keychain
    private class func readTunnelSettings(keychainReference: Data) -> Result<TunnelSettings, PacketTunnelProviderError> {
        return TunnelSettingsManager.load(searchTerm: .persistentReference(keychainReference))
            .mapError { PacketTunnelProviderError.cannotReadTunnelSettings($0) }
            .map { $0.tunnelSettings }
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private class func selectRelayEndpoint(relayConstraints: RelayConstraints) -> Result<RelaySelectorResult, PacketTunnelProviderError> {
        let cacheFileURL = RelayCache.IO.defaultCacheFileURL(forSecurityApplicationGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier)!
        let prebundledRelaysURL = RelayCache.IO.preBundledRelaysFileURL!

        return RelayCache.IO.readWithFallback(cacheFileURL: cacheFileURL, preBundledRelaysFileURL: prebundledRelaysURL)
            .mapError { relayCacheError -> PacketTunnelProviderError in
                return .readRelayCache(relayCacheError)
            }
            .flatMap { cachedRelayList -> Result<RelaySelectorResult, PacketTunnelProviderError> in
                if let selectorResult = RelaySelector.evaluate(relays: cachedRelayList.relays, constraints: relayConstraints) {
                    return .success(selectorResult)
                } else {
                    return .failure(.noRelaySatisfyingConstraint)
                }
            }
    }
}

enum PacketTunnelProviderError: ChainedError {
    /// Failure to read the relay cache
    case readRelayCache(RelayCache.Error)

    /// Failure to satisfy the relay constraint
    case noRelaySatisfyingConstraint

    /// Missing the persistent keychain reference to the tunnel settings
    case missingKeychainConfigurationReference

    /// Failure to read the tunnel settings from Keychain
    case cannotReadTunnelSettings(TunnelSettingsManager.Error)

    /// Failure to start the Wireguard backend
    case startWireguardAdapter(WireGuardAdapterError)

    /// Failure to stop the Wireguard backend
    case stopWireguardAdapter(WireGuardAdapterError)

    /// Failure to update the Wireguard configuration
    case updateWireguardConfiguration(WireGuardAdapterError)

    /// IPC handler failure
    case ipcHandler(Error)

    var errorDescription: String? {
        switch self {
        case .readRelayCache:
            return "Failure to read the relay cache"

        case .noRelaySatisfyingConstraint:
            return "No relay satisfying the given constraint"

        case .missingKeychainConfigurationReference:
            return "Keychain configuration reference is not set on protocol configuration"

        case .cannotReadTunnelSettings:
            return "Failure to read tunnel settings"

        case .startWireguardAdapter:
            return "Failure to start the WireGuard adapter"

        case .stopWireguardAdapter:
            return "Failure to stop the WireGuard adapter"

        case .updateWireguardConfiguration:
            return "Failure to update the Wireguard configuration"

        case .ipcHandler:
            return "Failure to handle the IPC request"
        }
    }
}

struct PacketTunnelConfiguration {
    var tunnelSettings: TunnelSettings
    var selectorResult: RelaySelectorResult
}

extension PacketTunnelConfiguration {

    var wgTunnelConfig: TunnelConfiguration {
        let mullvadEndpoint = selectorResult.endpoint
        var peers = [mullvadEndpoint.ipv4RelayEndpoint]
        if let ipv6RelayEndpoint = mullvadEndpoint.ipv6RelayEndpoint {
            peers.append(ipv6RelayEndpoint)
        }

        let peerConfigs = peers.compactMap { (endpoint) -> PeerConfiguration in
            let pubKey = PublicKey(rawValue: selectorResult.endpoint.publicKey)!
            var peerConfig = PeerConfiguration(publicKey: pubKey)
            peerConfig.endpoint = endpoint
            peerConfig.allowedIPs = [
                IPAddressRange(from: "0.0.0.0/0")!,
                IPAddressRange(from: "::/0")!
            ]
            return peerConfig
        }

        var interfaceConfig = InterfaceConfiguration(privateKey: tunnelSettings.interface.privateKey.privateKey)
        interfaceConfig.listenPort = 0
        interfaceConfig.dns = dnsServers.map { DNSServer(address: $0) }
        interfaceConfig.addresses = tunnelSettings.interface.addresses

        return TunnelConfiguration(name: nil, interface: interfaceConfig, peers: peerConfigs)
    }

    var dnsServers: [IPAddress] {
        let mullvadEndpoint = selectorResult.endpoint
        let dnsSettings = tunnelSettings.interface.dnsSettings

        switch (dnsSettings.blockAdvertising, dnsSettings.blockTracking) {
        case (true, false):
            return [IPv4Address("100.64.0.1")!]
        case (false, true):
            return [IPv4Address("100.64.0.2")!]
        case (true, true):
            return [IPv4Address("100.64.0.3")!]
        case (false, false):
            return [mullvadEndpoint.ipv4Gateway, mullvadEndpoint.ipv6Gateway]
        }
    }
}

extension WireGuardLogLevel {
    var loggerLevel: Logger.Level {
        switch self {
        case .verbose:
            return .debug
        case .error:
            return .error
        }
    }
}

extension WireGuardAdapter {
    func start(tunnelConfiguration: TunnelConfiguration) -> Result<(), WireGuardAdapterError>.Promise {
        return Result<(), WireGuardAdapterError>.Promise { resolver in
            self.start(tunnelConfiguration: tunnelConfiguration) { error in
                resolver.resolve(value: error.map { .failure($0) } ?? .success(()))
            }
        }
    }

    func stop() -> Result<(), WireGuardAdapterError>.Promise {
        return Result<(), WireGuardAdapterError>.Promise { resolver in
            self.stop { error in
                resolver.resolve(value: error.map { .failure($0) } ?? .success(()))
            }
        }
    }

    func update(tunnelConfiguration: TunnelConfiguration) -> Result<(), WireGuardAdapterError>.Promise {
        return Result<(), WireGuardAdapterError>.Promise { resolver in
            self.update(tunnelConfiguration: tunnelConfiguration) { error in
                resolver.resolve(value: error.map { .failure($0) } ?? .success(()))
            }
        }
    }
}

extension WireGuardAdapterError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .cannotLocateTunnelFileDescriptor:
            return "Failure to locate tunnel file descriptor."

        case .invalidState:
            return "Failure to perform an operation in such state."

        case .dnsResolution(let resolutionErrors):
            let detailedErrorDescription = resolutionErrors
                .enumerated()
                .map { index, dnsResolutionError in
                    return "\(index): \(dnsResolutionError.address) \(dnsResolutionError.errorDescription ?? "???")"
                }
                .joined(separator: "\n")

            return "Failure to resolve endpoints:\n\(detailedErrorDescription)"

        case .setNetworkSettings:
            return "Failure to set network settings"

        case .startWireGuardBackend(let code):
            return "Failure to start WireGuard backend (error code: \(code))"
        }
    }
}

extension MullvadEndpoint {
    var ipv4RelayEndpoint: Endpoint {
        return Endpoint(host: .ipv4(ipv4Relay.ip), port: .init(integerLiteral: ipv4Relay.port))
    }

    var ipv6RelayEndpoint: Endpoint? {
        guard let ipv6Relay = ipv6Relay else { return nil }

        return Endpoint(host: .ipv6(ipv6Relay.ip), port: .init(integerLiteral: ipv6Relay.port))
    }
}

