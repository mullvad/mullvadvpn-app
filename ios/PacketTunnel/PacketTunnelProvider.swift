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

class PacketTunnelProvider: NEPacketTunnelProvider {

    enum OperationCategory {
        case exclusive
    }

    /// Tunnel provider logger
    private let logger: Logger

    /// Current tunnel state
    private var tunnelState: PacketTunnelState = .disconnected {
        didSet {
            logger.info("New tunnel state: \(String(reflecting: self.tunnelState))")
        }
    }

    /// Internal queue
    private let dispatchQueue = DispatchQueue(label: "net.mullvad.MullvadVPN.PacketTunnel", qos: .utility)

    private lazy var operationQueue: OperationQueue = {
        let operationQueue = OperationQueue()
        operationQueue.underlyingQueue = self.dispatchQueue
        return operationQueue
    }()

    private lazy var exclusivityController: ExclusivityController<OperationCategory> = {
        return ExclusivityController(operationQueue: self.operationQueue)
    }()

    override init() {
        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!)

        logger = Logger(label: "PacketTunnelProvider")

        let wireguardLogger = Logger(label: "WireGuard")
        WireguardDevice.setTunnelLogger(wireguardLogger)
    }

    // MARK: - Subclass

    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        logger.info("Start the tunnel")

        let operation = AsyncBlockOperation { (finish) in
            self.doStartTunnel { (result) in
                switch result {
                case .success:
                    self.logger.info("Started the tunnel")
                    completionHandler(nil)

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to start the tunnel")
                    completionHandler(error)
                }

                finish()
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        logger.info("Stop the tunnel. Reason: \(reason)")

        let operation = AsyncBlockOperation { (finish) in
            self.doStopTunnel { (result) in
                switch result {
                case .success:
                    self.logger.info("Stopped the tunnel")
                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to stop the tunnel")
                }
                finish()
            }
        }

        operation.addDidFinishBlockObserver { (op) in
            completionHandler()
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            let decodeResult = PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
                .mapError { PacketTunnelProviderError.ipcHandler($0) }

            switch decodeResult {
            case .success(let request):
                switch request {
                case .reloadTunnelSettings:
                    self.reloadTunnelSettings { (result) in
                        self.replyAppMessage(result.map { true }, completionHandler: completionHandler)
                    }

                case .tunnelInformation:
                    self.replyAppMessage(.success(self.tunnelState.tunnelConnectionInfo), completionHandler: completionHandler)
                }

            case .failure(let error):
                self.replyAppMessage(Result<String, PacketTunnelProviderError>.failure(error), completionHandler: completionHandler)
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
        self.tunnelState = .connecting(nil)

        makePacketTunnelConfig { (result) in
            guard case .success(let packetTunnelConfig) = result else {
                self.tunnelState = .disconnected

                completionHandler(result.map { _ in () })
                return
            }

            self.tunnelState = .connecting(packetTunnelConfig.selectorResult.tunnelConnectionInfo)

            self.updateNetworkSettings(packetTunnelConfig: packetTunnelConfig) { (result) in
                guard case .success = result else {
                    self.tunnelState = .disconnected

                    completionHandler(result)
                    return
                }

                self.startWireguardDevice(packetFlow: self.packetFlow, configuration: packetTunnelConfig.wireguardConfig) { (result) in
                    guard case .success(let device) = result else {
                        self.tunnelState = .disconnected

                        completionHandler(result.map { _ in () })
                        return
                    }

                    let persistentKeychainReference = packetTunnelConfig.persistentKeychainReference
                    let keyRotationManager = AutomaticKeyRotationManager(persistentKeychainReference: persistentKeychainReference, eventQueue: self.dispatchQueue)
                    keyRotationManager.eventHandler = { [weak self] (keyRotationEvent) in
                        guard let self = self else { return }

                        self.reloadTunnelSettings { (result) in
                            switch result {
                            case .success:
                                break

                            case .failure(let error):
                                self.logger.error(chainedError: error, message: "Failed to reload tunnel settings")
                            }
                        }
                    }

                    RelayCache.shared.startPeriodicUpdates(queue: self.dispatchQueue) {
                        keyRotationManager.startAutomaticRotation(queue: self.dispatchQueue) {
                            let context = PacketTunnelContext(
                                wireguardDevice: device,
                                keyRotationManager: keyRotationManager
                            )

                            self.tunnelState = .connected(packetTunnelConfig.selectorResult.tunnelConnectionInfo, context)

                            completionHandler(.success(()))
                        }
                    }
                }
            }
        }
    }

    private func doStopTunnel(completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        guard let context = self.tunnelState.context else {
            logger.warning("Cannot stop the tunnel in such state: \(self.tunnelState)")
            completionHandler(.failure(.invalidTunnelState))
            return
        }

        self.tunnelState = .disconnecting

        RelayCache.shared.stopPeriodicUpdates(queue: self.dispatchQueue) {
            context.keyRotationManager.stopAutomaticRotation(queue: self.dispatchQueue) {
                context.wireguardDevice.stop(queue: self.dispatchQueue) { (result) in
                    let result = result.mapError({ (error) -> PacketTunnelProviderError in
                        return .stopWireguardDevice(error)
                    })

                    self.tunnelState = .disconnected

                    completionHandler(result)
                }
            }
        }
    }

    private func doReloadTunnelSettings(completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        guard let context = self.tunnelState.context else {
            logger.warning("Cannot reload tunnel settings in such state: \(self.tunnelState)")
            completionHandler(.failure(.invalidTunnelState))
            return
        }

        logger.info("Reload tunnel settings")

        let priorTunnelState = self.tunnelState
        self.tunnelState = .reconnecting(nil, context)

        makePacketTunnelConfig { (result) in
            guard case .success(let packetTunnelConfig) = result else {
                self.tunnelState = priorTunnelState

                completionHandler(result.map { _ in () })
                return
            }

            self.tunnelState = .reconnecting(packetTunnelConfig.selectorResult.tunnelConnectionInfo, context)

            // Tell the system that the tunnel is about to reconnect with the new endpoint
            self.reasserting = true

            let finishReconnecting = { (result: Result<(), PacketTunnelProviderError>) in
                switch result {
                case .success:
                    self.tunnelState = .connected(packetTunnelConfig.selectorResult.tunnelConnectionInfo, context)
                case .failure:
                    self.tunnelState = priorTunnelState
                }

                // Tell the system that the tunnel has finished reconnecting
                self.reasserting = false

                completionHandler(result)
            }

            self.updateNetworkSettings(packetTunnelConfig: packetTunnelConfig) { (result) in
                guard case .success = result else {
                    finishReconnecting(result)
                    return
                }

                context.wireguardDevice.setConfiguration(packetTunnelConfig.wireguardConfig, queue: self.dispatchQueue) { (result) in
                    finishReconnecting(result.mapError { PacketTunnelProviderError.updateWireguardConfiguration($0) })
                }
            }
        }
    }

    // MARK: - Private

    private func replyAppMessage<T: Codable>(
        _ result: Result<T, PacketTunnelProviderError>,
        completionHandler: ((Data?) -> Void)?) {
        let result = result.flatMap { (response) -> Result<Data, PacketTunnelProviderError> in
            return PacketTunnelIpcHandler.encodeResponse(response: response)
                .mapError { PacketTunnelProviderError.ipcHandler($0) }
        }

        switch result {
        case .success(let data):
            completionHandler?(data)

        case .failure(let error):
            self.logger.error(chainedError: error)
            completionHandler?(nil)
        }
    }

    private func makePacketTunnelConfig(completionHandler: @escaping (Result<PacketTunnelConfiguration, PacketTunnelProviderError>) -> Void) {
        guard let keychainReference = protocolConfiguration.passwordReference else {
            completionHandler(.failure(.missingKeychainConfigurationReference))
            return
        }

        Self.makePacketTunnelConfig(keychainReference: keychainReference, queue: self.dispatchQueue) { (result) in
            completionHandler(result)
        }
    }

    private func updateNetworkSettings(packetTunnelConfig: PacketTunnelConfiguration, completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        let settingsGenerator = PacketTunnelSettingsGenerator(
            mullvadEndpoint: packetTunnelConfig.selectorResult.endpoint,
            tunnelSettings: packetTunnelConfig.tunnelSettings
        )

        logger.info("Updating network settings...")

        setTunnelNetworkSettings(settingsGenerator.networkSettings()) { (error) in
            self.dispatchQueue.async {
                if let error = error {
                    self.logger.error("Cannot update network settings: \(error.localizedDescription)")

                    completionHandler(.failure(.setNetworkSettings(error)))
                } else {
                    self.logger.info("Updated network settings")

                    completionHandler(.success(()))
                }
            }
        }
    }

    private func reloadTunnelSettings(completionHandler: @escaping (Result<(), PacketTunnelProviderError>) -> Void) {
        let operation = AsyncBlockOperation { (finish) in
            self.doReloadTunnelSettings { (result) in
                completionHandler(result)
                finish()
            }
        }

        exclusivityController.addOperation(operation, categories: [.exclusive])
    }

    /// Returns a `PacketTunnelConfig` that contains the tunnel settings and selected relay
    private class func makePacketTunnelConfig(keychainReference: Data, queue: DispatchQueue?, completionHandler: @escaping (Result<PacketTunnelConfiguration, PacketTunnelProviderError>) -> Void) {
        switch Self.readTunnelSettings(keychainReference: keychainReference) {
        case .success(let tunnelSettings):
            Self.selectRelayEndpoint(relayConstraints: tunnelSettings.relayConstraints) { (result) in
                let result = result.map { (selectorResult) -> PacketTunnelConfiguration in
                    return PacketTunnelConfiguration(
                        persistentKeychainReference: keychainReference,
                        tunnelSettings: tunnelSettings,
                        selectorResult: selectorResult
                    )
                }

                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(result)
                }
            }

        case .failure(let error):
            queue.performOnWrappedOrCurrentQueue {
                completionHandler(.failure(error))
            }
        }
    }

    /// Read tunnel settings from Keychain
    private class func readTunnelSettings(keychainReference: Data) -> Result<TunnelSettings, PacketTunnelProviderError> {
        TunnelSettingsManager.load(searchTerm: .persistentReference(keychainReference))
            .mapError { PacketTunnelProviderError.cannotReadTunnelSettings($0) }
            .map { $0.tunnelSettings }
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private class func selectRelayEndpoint(relayConstraints: RelayConstraints, completionHandler: @escaping (Result<RelaySelectorResult, PacketTunnelProviderError>) -> Void) {
        RelayCache.shared.read { (result) in
            switch result {
            case .success(let cachedRelayList):
                let relaySelector = RelaySelector(relays: cachedRelayList.relays)

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

    private func startWireguardDevice(packetFlow: NEPacketTunnelFlow, configuration: WireguardConfiguration, completionHandler: @escaping (Result<WireguardDevice, PacketTunnelProviderError>) -> Void) {
        let result = WireguardDevice.fromPacketFlow(packetFlow)

        guard case .success(let device) = result else {
            completionHandler(result.mapError { PacketTunnelProviderError.startWireguardDevice($0) })
            return
        }

        let tunnelDeviceName = device.getInterfaceName() ?? "unknown"

        logger.info("Tunnel interface is \(tunnelDeviceName)")

        device.start(queue: dispatchQueue, configuration: configuration) { (result) in
            let result = result.map { device }
                .mapError { PacketTunnelProviderError.startWireguardDevice($0) }

            completionHandler(result)
        }
    }
}

enum PacketTunnelProviderError: ChainedError {
    /// Failure to perform operation in such state
    case invalidTunnelState

    /// Failure to read the relay cache
    case readRelayCache(RelayCacheError)

    /// Failure to satisfy the relay constraint
    case noRelaySatisfyingConstraint

    /// Missing the persistent keychain reference to the tunnel settings
    case missingKeychainConfigurationReference

    /// Failure to read the tunnel settings from Keychain
    case cannotReadTunnelSettings(TunnelSettingsManager.Error)

    /// Failure to set network settings
    case setNetworkSettings(Error)

    /// Failure to start the Wireguard backend
    case startWireguardDevice(WireguardDevice.Error)

    /// Failure to stop the Wireguard backend
    case stopWireguardDevice(WireguardDevice.Error)

    /// Failure to update the Wireguard configuration
    case updateWireguardConfiguration(Error)

    /// IPC handler failure
    case ipcHandler(PacketTunnelIpcHandler.Error)

    var errorDescription: String? {
        switch self {
        case .invalidTunnelState:
            return "Failure to handle request in such tunnel state"

        case .readRelayCache:
            return "Failure to read the relay cache"

        case .noRelaySatisfyingConstraint:
            return "No relay satisfying the given constraint"

        case .missingKeychainConfigurationReference:
            return "Keychain configuration reference is not set on protocol configuration"

        case .cannotReadTunnelSettings:
            return "Failure to read tunnel settings"

        case .setNetworkSettings:
            return "Failure to set system network settings"

        case .startWireguardDevice:
            return "Failure to start the WireGuard device"

        case .stopWireguardDevice:
            return "Failure to stop the WireGuard device"

        case .updateWireguardConfiguration:
            return "Failure to update the Wireguard configuration"

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

struct PacketTunnelContext {
    let wireguardDevice: WireguardDevice
    let keyRotationManager: AutomaticKeyRotationManager
}

enum PacketTunnelState {
    case connecting(TunnelConnectionInfo?)
    case connected(TunnelConnectionInfo, PacketTunnelContext)
    case disconnecting
    case disconnected
    case reconnecting(TunnelConnectionInfo?, PacketTunnelContext)

    var tunnelConnectionInfo: TunnelConnectionInfo? {
        switch self {
        case .connecting(let connectionInfo):
            return connectionInfo
        case .connected(let connectionInfo, _):
            return connectionInfo
        case .disconnecting:
            return nil
        case .disconnected:
            return nil
        case .reconnecting(let connectionInfo, _):
            return connectionInfo
        }
    }

    var context: PacketTunnelContext? {
        switch self {
        case .connecting:
            return nil
        case .connected(_, let context):
            return context
        case .disconnecting:
            return nil
        case .disconnected:
            return nil
        case .reconnecting(_, let context):
            return context
        }
    }
}

extension PacketTunnelState: CustomStringConvertible, CustomDebugStringConvertible {
    var description: String {
        switch self {
        case .connecting:
            return "connecting"
        case .connected:
            return "connected"
        case .disconnecting:
            return "disconnecting"
        case .disconnected:
            return "disconnected"
        case .reconnecting:
            return "reconnecting"
        }
    }

    var debugDescription: String {
        var output = "PacketTunnelState."

        switch self {
        case .connecting(let connectionInfo):
            output.append("connecting(")
            output.append(String(reflecting: connectionInfo))
            output.append(")")

        case .connected(let connectionInfo, _):
            output.append("connected(")
            output.append(String(reflecting: connectionInfo))
            output.append(")")

        case .disconnecting:
            output.append("disconnecting")

        case .disconnected:
            output.append("disconnected")

        case .reconnecting(let connectionInfo, _):
            output.append("reconnecting(")
            output.append(String(reflecting: connectionInfo))
            output.append(")")
        }

        return output
    }
}

extension RelaySelectorResult {
    var tunnelConnectionInfo: TunnelConnectionInfo {
        return TunnelConnectionInfo(
            ipv4Relay: self.endpoint.ipv4Relay,
            ipv6Relay: self.endpoint.ipv6Relay,
            hostname: self.relay.hostname,
            location: self.location
        )
    }
}

