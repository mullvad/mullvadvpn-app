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

class PacketTunnelProvider: NEPacketTunnelProvider, TunnelMonitorDelegate {

    /// Tunnel provider logger.
    private let providerLogger: Logger

    /// WireGuard adapter logger.
    private let tunnelLogger: Logger

    /// Internal queue.
    private let dispatchQueue = DispatchQueue(label: "PacketTunnel", qos: .utility)

    /// WireGuard adapter.
    private var adapter: WireGuardAdapter!

    /// Raised once tunnel establishes connection in the very first time, before calling the system
    /// completion handler passed into `startTunnel`.
    private var isConnected = false

    /// A system completion handler passed from startTunnel and saved for later use once the
    /// connection is established.
    private var startTunnelCompletionHandler: ((PacketTunnelProviderError?) -> Void)?

    /// A completion handler passed during reassertion and saved for later use once the connection
    /// is reestablished.
    private var reassertTunnelCompletionHandler: ((PacketTunnelProviderError?) -> Void)?

    /// Tunnel monitor.
    private var tunnelMonitor: TunnelMonitor!

    /// Tunnel status.
    private var tunnelStatus = PacketTunnelStatus(
        isNetworkReachable: true,
        connectingDate: nil,
        tunnelRelay: nil
    )

    override init() {
        let pid = ProcessInfo.processInfo.processIdentifier

        var metadata = Logger.Metadata()
        metadata["pid"] = .string("\(pid)")

        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!, metadata: metadata)

        providerLogger = Logger(label: "PacketTunnelProvider")
        tunnelLogger = Logger(label: "WireGuard")

        super.init()

        adapter = WireGuardAdapter(with: self, shouldHandleReasserting: false, logHandler: { [weak self] (logLevel, message) in
            self?.dispatchQueue.async {
                self?.tunnelLogger.log(level: logLevel.loggerLevel, "\(message)")
            }
        })

        tunnelMonitor = TunnelMonitor(queue: dispatchQueue, adapter: adapter)
        tunnelMonitor.delegate = self
    }

    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])
        var appSelectorResult: RelaySelectorResult?

        // Parse relay selector from tunnel options.
        do {
            appSelectorResult = try tunnelOptions.getSelectorResult()

            switch appSelectorResult {
            case .some(let selectorResult):
                providerLogger.debug("Start the tunnel via app, connect to \(selectorResult.relay.hostname).")

            case .none:
                if tunnelOptions.isOnDemand() {
                    providerLogger.debug("Start the tunnel via on-demand rule.")
                } else {
                    providerLogger.debug("Start the tunnel via system.")
                }
            }
        } catch {
            providerLogger.debug("Start the tunnel via app.")
            providerLogger.error(
                chainedError: AnyChainedError(error),
                message: "Failed to decode relay selector result passed from the app. Will continue by picking new relay."
            )
        }

        // Read tunnel configuration.
        let tunnelConfiguration: PacketTunnelConfiguration
        switch makeConfiguration(appSelectorResult) {
        case .success(let configuration):
            tunnelConfiguration = configuration

        case .failure(let error):
            providerLogger.error(chainedError: error, message: "Failed to start the tunnel.")
            completionHandler(error)
            return
        }

        // Set tunnel status.
        dispatchQueue.async {
            let tunnelRelay = tunnelConfiguration.selectorResult.packetTunnelRelay
            self.tunnelStatus.tunnelRelay = tunnelRelay
            self.providerLogger.debug("Set tunnel relay to \(tunnelRelay.hostname).")
        }

        // Start tunnel.
        adapter.start(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
            self.dispatchQueue.async {
                if let error = error {
                    let tunnelProviderError = PacketTunnelProviderError.startWireguardAdapter(error)
                    self.providerLogger.error(chainedError: tunnelProviderError, message: "Failed to start the tunnel.")

                    completionHandler(tunnelProviderError)
                } else {
                    self.providerLogger.debug("Started the tunnel.")

                    // Store completion handler and call it from TunnelMonitorDelegate once
                    // the connection is established.
                    self.startTunnelCompletionHandler = { [weak self] error in
                        // Mark the tunnel connected.
                        self?.isConnected = true

                        // Call system completion handler.
                        completionHandler(error)
                    }

                    // Start tunnel monitor.
                    let gatewayAddress = tunnelConfiguration.selectorResult.endpoint.ipv4Gateway

                    self.startTunnelMonitor(gatewayAddress: gatewayAddress)
                }
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        providerLogger.debug("Stop the tunnel: \(reason)")

        dispatchQueue.async {
            // Stop tunnel monitor.
            self.tunnelMonitor.stop()

            // Unset the start tunnel completion handler.
            self.startTunnelCompletionHandler = nil
        }

        adapter.stop { error in
            self.dispatchQueue.async {
                let tunnelProviderError = error.map { PacketTunnelProviderError.stopWireguardAdapter($0) }

                if let tunnelProviderError = tunnelProviderError {
                    self.providerLogger.error(chainedError: tunnelProviderError, message: "Failed to stop the tunnel gracefully.")
                }

                self.providerLogger.debug("Stopped the tunnel.")
                completionHandler()
            }
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            let request: TunnelIPC.Request
            do {
                request = try TunnelIPC.Coding.decodeRequest(messageData)
            } catch {
                self.providerLogger.error(chainedError: AnyChainedError(error), message: "Failed to decode the app message request.")

                completionHandler?(nil)
                return
            }

            self.providerLogger.debug("Received app message: \(request)")

            switch request {
            case .reloadTunnelSettings:
                self.providerLogger.debug("Reloading tunnel settings...")

                self.reloadTunnelSettings { [weak self] error in
                    guard let self = self else { return }

                    if let error = error {
                        self.providerLogger.error(chainedError: error, message: "Failed to reload tunnel settings.")
                    } else {
                        self.providerLogger.debug("Reloaded tunnel settings.")
                    }
                }

                completionHandler?(nil)

            case .getTunnelStatus:
                var response: Data?
                do {
                    response = try TunnelIPC.Coding.encodeResponse(self.tunnelStatus)
                } catch {
                    self.providerLogger.error(chainedError: AnyChainedError(error), message: "Failed to encode the app message response for \(request)")
                }

                completionHandler?(response)
            }
        }
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        // Add code here to get ready to sleep.
        completionHandler()
    }

    override func wake() {
        // Add code here to wake up.
    }

    // MARK: - TunnelMonitor

    func tunnelMonitorDidDetermineConnectionEstablished(_ tunnelMonitor: TunnelMonitor) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Connection established.")

        tunnelStatus.connectingDate = nil

        startTunnelCompletionHandler?(nil)
        startTunnelCompletionHandler = nil

        reassertTunnelCompletionHandler?(nil)
        reassertTunnelCompletionHandler = nil
    }

    func tunnelMonitorDelegateShouldHandleConnectionRecovery(_ tunnelMonitor: TunnelMonitor) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Recover connection. Picking next relay...")

        let handleRecoveryFailure = { (_ error: PacketTunnelProviderError) in
            // Stop tunnel monitor.
            tunnelMonitor.stop()

            // Call start tunnel completion handler with error.
            self.startTunnelCompletionHandler?(error)

            // Reset start tunnel completion handler.
            self.startTunnelCompletionHandler = nil

            // Call tunnel reassertion completion handler with error.
            self.reassertTunnelCompletionHandler?(error)

            // Reset tunnel reassertion completion handler.
            self.reassertTunnelCompletionHandler = nil
        }

        // Read tunnel configuration.
        let tunnelConfiguration: PacketTunnelConfiguration
        switch makeConfiguration(nil) {
        case .success(let configuration):
            tunnelConfiguration = configuration

        case .failure(let error):
            handleRecoveryFailure(error)
            return
        }

        // Update tunnel status.
        let tunnelRelay = tunnelConfiguration.selectorResult.packetTunnelRelay
        tunnelStatus.tunnelRelay = tunnelRelay
        providerLogger.debug("Set tunnel relay to \(tunnelRelay.hostname).")

        // Update WireGuard configuration.
        adapter.update(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
            self.dispatchQueue.async {
                if let error = error {
                    handleRecoveryFailure(.updateWireguardConfiguration(error))
                }
            }
        }
    }

    func tunnelMonitor(_ tunnelMonitor: TunnelMonitor, networkReachabilityStatusDidChange isNetworkReachable: Bool) {
        tunnelStatus.isNetworkReachable = isNetworkReachable

        // Adjust the start reconnect date if tunnel monitor re-started pinging in response to
        // network connectivity coming back up.
        if let startDate = tunnelMonitor.startDate {
            tunnelStatus.connectingDate = startDate
        }
    }

    // MARK: - Private

    private func makeConfiguration(_ appSelectorResult: RelaySelectorResult? = nil) -> Result<PacketTunnelConfiguration, PacketTunnelProviderError> {
        guard let passwordRef = protocolConfiguration.passwordReference else {
            return .failure(.missingKeychainConfigurationReference)
        }

        let keychainEntry: TunnelSettingsManager.KeychainEntry
        switch TunnelSettingsManager.load(searchTerm: .persistentReference(passwordRef)) {
        case .success(let entry):
            keychainEntry = entry
        case .failure(let error):
            return .failure(.cannotReadTunnelSettings(error))
        }

        let selectorResult: RelaySelectorResult
        if let appSelectorResult = appSelectorResult {
            selectorResult = appSelectorResult
        } else {
            let relayConstraints = keychainEntry.tunnelSettings.relayConstraints
            switch Self.selectRelayEndpoint(relayConstraints: relayConstraints) {
            case .success(let value):
                selectorResult = value
            case .failure(let error):
                return .failure(error)
            }
        }

        let tunnelConfiguration = PacketTunnelConfiguration(
            tunnelSettings: keychainEntry.tunnelSettings,
            selectorResult: selectorResult
        )

        return .success(tunnelConfiguration)
    }

    private func reloadTunnelSettings(completionHandler: @escaping (PacketTunnelProviderError?) -> Void) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        // Read tunnel configuration.
        let tunnelConfiguration: PacketTunnelConfiguration
        switch makeConfiguration(nil) {
        case .success(let configuration):
            tunnelConfiguration = configuration

        case .failure(let error):
            completionHandler(error)
            return
        }

        // Copy old relay.
        let oldTunnelRelay = tunnelStatus.tunnelRelay
        let newTunnelRelay = tunnelConfiguration.selectorResult.packetTunnelRelay

        // Update tunnel status.
        tunnelStatus.tunnelRelay = newTunnelRelay
        tunnelStatus.connectingDate = nil

        providerLogger.debug("Set tunnel relay to \(newTunnelRelay.hostname).")

        // Raise reasserting flag, but only if tunnel has already moved to connected state once.
        // Otherwise keep the app in connecting state until it manages to establish the very first
        // connection.
        if isConnected {
            reasserting = true
        }

        // Update WireGuard configuration.
        adapter.update(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
            self.dispatchQueue.async {
                // Reset previously stored completion handler.
                self.reassertTunnelCompletionHandler = nil

                // Call completion handler immediately on error to update adapter configuration.
                if let error = error {
                    // Revert to previously used tunnel relay.
                    self.tunnelStatus.tunnelRelay = oldTunnelRelay
                    self.providerLogger.debug("Reset tunnel relay to \(oldTunnelRelay?.hostname ?? "none").")

                    // Lower the reasserting flag.
                    if self.isConnected {
                        self.reasserting = false
                    }

                    // Call completion handler immediately.
                    completionHandler(.updateWireguardConfiguration(error))
                } else {
                    // Store completion handler and call it from TunnelMonitorDelegate once
                    // the connection is established.
                    self.reassertTunnelCompletionHandler = { [weak self] providerError in
                        guard let self = self else { return }

                        // Lower the reasserting flag.
                        if self.isConnected {
                            self.reasserting = false
                        }

                        completionHandler(providerError)
                    }

                    // Restart tunnel monitor.
                    let gatewayAddress = tunnelConfiguration.selectorResult.endpoint.ipv4Gateway

                    self.startTunnelMonitor(gatewayAddress: gatewayAddress)
                }
            }
        }
    }

    private func startTunnelMonitor(gatewayAddress: IPv4Address) {
        tunnelMonitor.start(address: gatewayAddress)

        // Mark when the tunnel started monitoring connection.
        tunnelStatus.connectingDate = tunnelMonitor.startDate
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private class func selectRelayEndpoint(relayConstraints: RelayConstraints) -> Result<RelaySelectorResult, PacketTunnelProviderError> {
        let cacheFileURL = RelayCache.IO.defaultCacheFileURL(forSecurityApplicationGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier)!
        let prebundledRelaysURL = RelayCache.IO.preBundledRelaysFileURL!

        return RelayCache.IO.readWithFallback(cacheFileURL: cacheFileURL, preBundledRelaysFileURL: prebundledRelaysURL)
            .mapError { error in
                return PacketTunnelProviderError.readRelayCache(error)
            }
            .flatMap { cachedRelayList in
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

    var errorDescription: String? {
        switch self {
        case .readRelayCache:
            return "Failure to read the relay cache."

        case .noRelaySatisfyingConstraint:
            return "No relay satisfying the given constraint."

        case .missingKeychainConfigurationReference:
            return "Keychain configuration reference is not set on protocol configuration."

        case .cannotReadTunnelSettings:
            return "Failure to read tunnel settings."

        case .startWireguardAdapter:
            return "Failure to start the WireGuard adapter."

        case .stopWireguardAdapter:
            return "Failure to stop the WireGuard adapter."

        case .updateWireguardConfiguration:
            return "Failure to update the Wireguard configuration."
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

        if dnsSettings.effectiveEnableCustomDNS {
            let dnsServers = dnsSettings.customDNSDomains
                .prefix(DNSSettings.maxAllowedCustomDNSDomains)
            return Array(dnsServers)
        } else {
            if let serverAddress = dnsSettings.blockingOptions.serverAddress {
                return [serverAddress]
            } else {
                return [mullvadEndpoint.ipv4Gateway, mullvadEndpoint.ipv6Gateway]
            }
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
            return "Failure to set network settings."

        case .startWireGuardBackend(let code):
            return "Failure to start WireGuard backend (error code: \(code))."
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
