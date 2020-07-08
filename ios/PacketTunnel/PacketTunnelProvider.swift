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
import os

enum PacketTunnelProviderError: ChainedError {
    /// Failure to read the relay cache
    case readRelayCache(RelayCacheError)

    /// Failure to satisfy the relay constraint
    case noRelaySatisfyingConstraint

    /// Missing the persistent keychain reference to the tunnel configuration
    case missingKeychainConfigurationReference

    /// Failure to read the tunnel configuration from Keychain
    case cannotReadTunnelConfiguration(TunnelSettingsManager.Error)

    /// Failure to set network settings
    case setNetworkSettings(Error)

    /// Failure to start the Wireguard backend
    case startWireguardDevice(WireguardDevice.Error)

    /// Failure to update the Wireguard configuration
    case updateWireguardConfiguration(Error)

    /// IPC handler failure
    case ipcHandler(PacketTunnelIpcHandler.Error)

    var errorDescription: String? {
        switch self {
        case .readRelayCache:
            return "Failure to read the relay cache"

        case .noRelaySatisfyingConstraint:
            return "No relay satisfying the given constraint"

        case .missingKeychainConfigurationReference:
            return "Invalid protocol configuration"

        case .cannotReadTunnelConfiguration:
            return "Failure reading tunnel configuration"

        case .setNetworkSettings:
            return "Failure to set system network settings"

        case .startWireguardDevice:
            return "Failure starting WireGuard device"

        case .updateWireguardConfiguration:
            return "Failure to update Wireguard configuration"

        case .ipcHandler:
            return "Failure to handle the IPC request"
        }
    }
}

struct PacketTunnelConfiguration {
    var persistentKeychainReference: Data
    var tunnelSettings: TunnelSettings
    var selectorResult: RelaySelectorResult
}

extension PacketTunnelConfiguration {
    var wireguardConfig: WireguardConfiguration {
        let mullvadEndpoint = selectorResult.endpoint
        var peers: [AnyIPEndpoint] = [.ipv4(mullvadEndpoint.ipv4Relay)]

        if let ipv6Relay = mullvadEndpoint.ipv6Relay {
            peers.append(.ipv6(ipv6Relay))
        }

        let wireguardPeers = peers.map {
            WireguardPeer(
                endpoint: $0,
                publicKey: selectorResult.endpoint.publicKey)
        }

        return WireguardConfiguration(
            privateKey: tunnelSettings.interface.privateKey,
            peers: wireguardPeers,
            allowedIPs: [
                IPAddressRange(address: IPv4Address.any, networkPrefixLength: 0),
                IPAddressRange(address: IPv6Address.any, networkPrefixLength: 0)
            ]
        )
    }
}

class PacketTunnelProvider: NEPacketTunnelProvider {

    enum OperationCategory {
        case exclusive
    }

    /// Active wireguard device
    private var wireguardDevice: WireguardDevice?

    /// Active tunnel connection information
    private var connectionInfo: TunnelConnectionInfo?

    /// The completion handler to call when the tunnel is fully established
    private var pendingStartCompletion: ((Error?) -> Void)?

    private let dispatchQueue = DispatchQueue(label: "net.mullvad.MullvadVPN.PacketTunnel", qos: .utility)

    private lazy var operationQueue: OperationQueue = {
        let operationQueue = OperationQueue()
        operationQueue.underlyingQueue = self.dispatchQueue
        return operationQueue
    }()
    private lazy var exclusivityController: ExclusivityController<OperationCategory> = {
        return ExclusivityController(operationQueue: self.operationQueue)
    }()

    private var keyRotationManager: AutomaticKeyRotationManager?

    override init() {
        super.init()

        self.configureLogger()
    }

    // MARK: - Subclass

    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        dispatchQueue.async {
            let operation = AsyncBlockOperation { (finish) in
                os_log(.default, log: tunnelProviderLog, "Start the tunnel")

                self.doStartTunnel { (result) in
                    switch result {
                    case .success:
                        self.pendingStartCompletion?(nil)

                    case .failure(let error):
                        error.logChain(log: tunnelProviderLog)
                        self.pendingStartCompletion?(error)
                    }

                    finish()
                }
            }

            self.pendingStartCompletion = completionHandler
            self.exclusivityController.addOperation(operation, categories: [.exclusive])
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        let operation = AsyncBlockOperation { (finish) in
            os_log(.default, log: tunnelProviderLog, "Stop the tunnel. Reason: %{public}s", "\(reason)")

            self.doStopTunnel {
                completionHandler()
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            let finishWithResult = { (result: Result<AnyEncodable, PacketTunnelProviderError>) in
                let result = result.flatMap { (response) -> Result<Data, PacketTunnelProviderError> in
                    return PacketTunnelIpcHandler.encodeResponse(response: response)
                        .mapError { PacketTunnelProviderError.ipcHandler($0) }
                }

                switch result {
                case .success(let data):
                    completionHandler?(data)

                case .failure(let error):
                    error.logChain(log: tunnelProviderLog)
                    completionHandler?(nil)
                }
            }

            let decodeResult = PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
                .mapError { PacketTunnelProviderError.ipcHandler($0) }

            switch decodeResult {
            case .success(let request):
                switch request {
                case .reloadTunnelSettings:
                    self.reloadTunnelSettings { (result) in
                        finishWithResult(result.map { AnyEncodable(true) })
                    }

                case .tunnelInformation:
                    finishWithResult(.success(AnyEncodable(self.connectionInfo)))
                }

            case .failure(let error):
                finishWithResult(.failure(error))
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

    // MARK: - Tunnel management

    private func doStartTunnel(completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        makePacketTunnelConfig { (result) in
            guard case .success(let packetTunnelConfig) = result else {
                completionHandler(result.map { _ in () })
                return
            }

            self.updateNetworkSettings(packetTunnelConfig: packetTunnelConfig) { (result) in
                guard case .success = result else {
                    completionHandler(result)
                    return
                }

                Self.startWireguardDevice(packetFlow: self.packetFlow, configuration: packetTunnelConfig.wireguardConfig) { (result) in
                    self.dispatchQueue.async {
                        guard case .success(let device) = result else {
                            completionHandler(result.map { _ in () })
                            return
                        }

                        RelayCache.shared.startPeriodicUpdates(completionHandler: nil)

                        let persistentKeychainReference = packetTunnelConfig.persistentKeychainReference
                        let keyRotationManager = AutomaticKeyRotationManager(persistentKeychainReference: persistentKeychainReference)

                        keyRotationManager.eventHandler = { (keyRotationEvent) in
                            self.dispatchQueue.async {
                                self.reloadTunnelSettings { (result) in
                                    switch result {
                                    case .success:
                                        break

                                    case .failure(let error):
                                        error.logChain(message: "Failed to reload tunnel settings", log: tunnelProviderLog)
                                    }
                                }
                            }
                        }

                        self.wireguardDevice = device
                        self.keyRotationManager = keyRotationManager

                        keyRotationManager.startAutomaticRotation {
                            completionHandler(.success(()))
                        }
                    }
                }
            }
        }
    }

    private func doStopTunnel(completionHandler: @escaping () -> Void) {
        guard let device = self.wireguardDevice, let keyRotationManager = self.keyRotationManager
            else {
                completionHandler()
                return
        }

        RelayCache.shared.stopPeriodicUpdates(completionHandler: nil)

        keyRotationManager.stopAutomaticRotation {
            device.stop { (result) in
                self.dispatchQueue.async {
                    self.wireguardDevice = nil
                    self.keyRotationManager = nil

                    if case .failure(let error) = result {
                        error.logChain(message: "Failed to stop the tunnel", log: tunnelProviderLog)
                    }

                    // Ignore all errors at this point
                    completionHandler()
                }
            }
        }
    }

    private func doReloadTunnelSettings(completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        guard let device = self.wireguardDevice else {
            os_log(.default, log: tunnelProviderLog, "Ignore reloading tunnel settings. The WireguardDevice is not set yet.")

            completionHandler(.success(()))
            return
        }

        os_log(.default, log: tunnelProviderLog, "Reload tunnel settings")

        makePacketTunnelConfig { (result) in
            guard case .success(let packetTunnelConfig) = result else {
                completionHandler(result.map { _ in () })
                return
            }

            // Tell the system that the tunnel is about to reconnect with the new endpoint
            self.reasserting = true

            let finishReconnecting = { (result: Result<(), PacketTunnelProviderError>) in
                // Tell the system that the tunnel has finished reconnecting
                self.reasserting = false

                completionHandler(result)
            }

            self.updateNetworkSettings(packetTunnelConfig: packetTunnelConfig) { (result) in
                guard case .success = result else {
                    finishReconnecting(result)
                    return
                }

                device.setConfiguration(packetTunnelConfig.wireguardConfig) { (result) in
                    self.dispatchQueue.async {
                        finishReconnecting(result.mapError { PacketTunnelProviderError.updateWireguardConfiguration($0) })
                    }
                }
            }
        }
    }

    // MARK: - Private

    private func configureLogger() {
        WireguardDevice.setLogger { (level, message) in
            os_log(level.osLogType, log: wireguardLog, "%{public}s", message)
        }
    }

    private func setTunnelConnectionInfo(selectorResult: RelaySelectorResult) {
        self.connectionInfo = TunnelConnectionInfo(
            ipv4Relay: selectorResult.endpoint.ipv4Relay,
            ipv6Relay: selectorResult.endpoint.ipv6Relay,
            hostname: selectorResult.relay.hostname,
            location: selectorResult.location
        )

        os_log(.default, log: tunnelProviderLog, "Select relay: %{public}s",
               selectorResult.relay.hostname)
    }

    private func makePacketTunnelConfig(completionHandler: @escaping (Result<PacketTunnelConfiguration, PacketTunnelProviderError>) -> Void) {
        guard let keychainReference = protocolConfiguration.passwordReference else {
            completionHandler(.failure(.missingKeychainConfigurationReference))
            return
        }

        Self.makePacketTunnelConfig(keychainReference: keychainReference) { (result) in
            self.dispatchQueue.async {
                guard case .success(let packetTunnelConfig) = result else {
                    completionHandler(result)
                    return
                }

                self.setTunnelConnectionInfo(selectorResult: packetTunnelConfig.selectorResult)

                os_log(.default, log: tunnelProviderLog, "Set tunnel network settings")

                completionHandler(result)
            }
        }
    }

    private func updateNetworkSettings(packetTunnelConfig: PacketTunnelConfiguration, completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        let settingsGenerator = PacketTunnelSettingsGenerator(
            mullvadEndpoint: packetTunnelConfig.selectorResult.endpoint,
            tunnelConfiguration: packetTunnelConfig.tunnelSettings
        )

        setTunnelNetworkSettings(settingsGenerator.networkSettings()) { (error) in
            self.dispatchQueue.async {
                if let error = error {
                    os_log(.error, log: tunnelProviderLog, "Cannot set network settings: %{public}s", error.localizedDescription)

                    completionHandler(.failure(.setNetworkSettings(error)))
                } else {
                    completionHandler(.success(()))
                }
            }
        }
    }

    private func reloadTunnelSettings(completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        let operation = ResultOperation<(), PacketTunnelProviderError> { (finish) in
            self.doReloadTunnelSettings { (result) in
                finish(result)
            }
        }

        operation.addDidFinishBlockObserver { (operation, result) in
            self.dispatchQueue.async {
                completionHandler(result)
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    /// Returns a `PacketTunnelConfig` that contains the tunnel configuration and selected relay
    private class func makePacketTunnelConfig(keychainReference: Data, completionHandler: @escaping (Result<PacketTunnelConfiguration, PacketTunnelProviderError>) -> Void) {
        switch Self.readTunnelConfiguration(keychainReference: keychainReference) {
        case .success(let tunnelSettings):
            Self.selectRelayEndpoint(relayConstraints: tunnelSettings.relayConstraints) { (result) in
                let result = result.map { (selectorResult) -> PacketTunnelConfiguration in
                    return PacketTunnelConfiguration(
                        persistentKeychainReference: keychainReference,
                        tunnelSettings: tunnelSettings,
                        selectorResult: selectorResult
                    )
                }
                completionHandler(result)
            }

        case .failure(let error):
            completionHandler(.failure(error))
        }
    }

    /// Read tunnel configuration from Keychain
    private class func readTunnelConfiguration(keychainReference: Data) -> Result<TunnelSettings, PacketTunnelProviderError> {
        TunnelSettingsManager.load(searchTerm: .persistentReference(keychainReference))
            .mapError { PacketTunnelProviderError.cannotReadTunnelConfiguration($0) }
            .map { $0.tunnelSettings }
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private class func selectRelayEndpoint(relayConstraints: RelayConstraints, completionHandler: @escaping (Result<RelaySelectorResult, PacketTunnelProviderError>) -> Void) {
        RelayCache.shared.read { (result) in
            switch result {
            case .success(let cachedRelayList):
                let relaySelector = RelaySelector(relayList: cachedRelayList.relayList)

                if let selectorResult = relaySelector.evaluate(with: relayConstraints) {
                    completionHandler(.success(selectorResult))
                } else {
                    completionHandler(.failure(.noRelaySatisfyingConstraint))
                }

            case .failure(let error):
                completionHandler(.failure(.readRelayCache(error)))
            }
        }
    }

    private class func startWireguardDevice(packetFlow: NEPacketTunnelFlow, configuration: WireguardConfiguration, completionHandler: @escaping (Result<WireguardDevice, PacketTunnelProviderError>) -> Void) {
        let result = WireguardDevice.fromPacketFlow(packetFlow)

        guard case .success(let device) = result else {
            completionHandler(result.mapError { PacketTunnelProviderError.startWireguardDevice($0) })
            return
        }

        let tunnelDeviceName = device.getInterfaceName() ?? "unknown"

        os_log(.default, log: tunnelProviderLog, "Tunnel interface is %{public}s", tunnelDeviceName)

        device.start(configuration: configuration) { (result) in
            let result = result.map { device }
                .mapError { PacketTunnelProviderError.startWireguardDevice($0) }

            completionHandler(result)
        }
    }
}
