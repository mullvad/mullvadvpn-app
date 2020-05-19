//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import Network
import NetworkExtension
import os

enum PacketTunnelProviderError: Error {
    /// Failure to read the relay cache
    case readRelayCache(RelayCacheError)

    /// Failure to satisfy the relay constraint
    case noRelaySatisfyingConstraint

    /// Missing the persistent keychain reference to the tunnel configuration
    case missingKeychainConfigurationReference

    /// Failure to read the tunnel configuration from Keychain
    case cannotReadTunnelConfiguration(TunnelConfigurationManager.Error)

    /// Failure to set network settings
    case setNetworkSettings(Error)

    /// Failure to start the Wireguard backend
    case startWireguardDevice(WireguardDevice.Error)

    /// Failure to update the Wireguard configuration
    case updateWireguardConfiguration(Error)

    /// IPC handler failure
    case ipcHandler(PacketTunnelIpcHandlerError)

    var localizedDescription: String {
        switch self {
        case .readRelayCache(let relayError):
            return "Failure to read the relay cache: \(relayError.localizedDescription)"

        case .noRelaySatisfyingConstraint:
            return "No relay satisfying the given constraint"

        case .missingKeychainConfigurationReference:
            return "Invalid protocol configuration"

        case .cannotReadTunnelConfiguration(let readError):
            return "Cannot read tunnel configuration: \(readError.localizedDescription)"

        case .setNetworkSettings(let systemError):
            return "Failed to set network settings: \(systemError.localizedDescription)"

        case .startWireguardDevice(let deviceError):
            return "Failure to start Wireguard: \(deviceError.localizedDescription)"

        case .updateWireguardConfiguration(let error):
            return "Failure to update Wireguard configuration: \(error.localizedDescription)"

        case .ipcHandler(let ipcError):
            return "Failure to handle the IPC request: \(ipcError.localizedDescription)"
        }
    }
}

struct PacketTunnelConfiguration {
    var persistentKeychainReference: Data
    var tunnelConfig: TunnelConfiguration
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
            privateKey: tunnelConfig.interface.privateKey,
            peers: wireguardPeers,
            allowedIPs: [
                IPAddressRange(address: IPv4Address.any, networkPrefixLength: 0),
                IPAddressRange(address: IPv6Address.any, networkPrefixLength: 0)
            ]
        )
    }
}

class PacketTunnelProvider: NEPacketTunnelProvider {

    /// Active wireguard device
    private var wireguardDevice: WireguardDevice?

    /// Active tunnel connection information
    private var connectionInfo: TunnelConnectionInfo?
    private let cancellableSet = CancellableSet()

    private var startStopTunnelSubscriber: AnyCancellable?
    private var startedTunnel = false

    private let exclusivityQueue = DispatchQueue(label: "net.mullvad.vpn.packet-tunnel.exclusivity-queue")
    private let executionQueue = DispatchQueue(label: "net.mullvad.vpn.packet-tunnel.execution-queue")

    private var keyRotationManager: AutomaticKeyRotationManager?

    override init() {
        super.init()

        self.configureLogger()
    }

    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        startStopTunnelSubscriber = self.startTunnel()
            .sink(receiveCompletion: { (completion) in
                switch completion {
                case .finished:
                    completionHandler(nil)

                case .failure(let error):
                    os_log(.error, log: tunnelProviderLog,
                           "Failed to start the tunnel: %{public}s", error.localizedDescription)

                    completionHandler(error)
                }
            })
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        startStopTunnelSubscriber = stopTunnel(reason: reason)
            .sink(receiveCompletion: { (completion) in
                completionHandler()
            })
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
            .mapError { PacketTunnelProviderError.ipcHandler($0) }
            .receive(on: executionQueue)
            .flatMap { (request) -> AnyPublisher<AnyEncodable, PacketTunnelProviderError> in
                os_log(.default, log: tunnelProviderLog, "IPC request: %{public}s", "\(request)")

                switch request {

                case .reloadConfiguration:
                    return self.reloadTunnel()
                        .map { AnyEncodable(true) }
                        .eraseToAnyPublisher()

                case .tunnelInformation:
                    return Result.Publisher(AnyEncodable(self.connectionInfo))
                        .eraseToAnyPublisher()

                }
        }.flatMap({ (response) in
            return PacketTunnelIpcHandler.encodeResponse(response: response)
                .mapError { PacketTunnelProviderError.ipcHandler($0) }
        }).autoDisposableSink(cancellableSet: cancellableSet, receiveCompletion: { (completion) in
            if case .failure(let error) = completion {
                os_log(.error, log: tunnelProviderLog,
                       "Failed to handle the app message: %{public}s", error.localizedDescription)
                completionHandler?(nil)
            }
        }, receiveValue: { (responseData) in
            completionHandler?(responseData)
        })
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        // Add code here to get ready to sleep.
        completionHandler()
    }

    override func wake() {
        // Add code here to wake up.
    }

    private func configureLogger() {
        WireguardDevice.setLogger { (level, message) in
            os_log(level.osLogType, log: wireguardLog, "%{public}s", message)
        }
    }

    private func startTunnel() -> AnyPublisher<(), PacketTunnelProviderError> {
        MutuallyExclusive(
            exclusivityQueue: exclusivityQueue,
            executionQueue: executionQueue
        ) { () -> AnyPublisher<(), PacketTunnelProviderError> in
            os_log(.default, log: tunnelProviderLog, "Start the tunnel")

            self.startedTunnel = true

            return self.makePacketTunnelConfigAndApplyNetworkSettings()
                .flatMap { (packetTunnelConfiguration) in
                    Self.startWireguard(
                        packetFlow: self.packetFlow,
                        configuration: packetTunnelConfiguration.wireguardConfig
                    )
                        .receive(on: self.executionQueue)
                        .handleEvents(receiveOutput: { (wireguardDevice) in
                            self.wireguardDevice = wireguardDevice

                            self.startKeyRotation(
                                persistentKeychainReference: packetTunnelConfiguration
                                    .persistentKeychainReference
                            )
                        }).map { _ in () }
            }.eraseToAnyPublisher()
        }.eraseToAnyPublisher()
    }

    private func stopTunnel(reason: NEProviderStopReason) -> AnyPublisher<(), Never> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) { () -> AnyPublisher<(), Never> in
            os_log(.default, log: tunnelProviderLog,
                   "Stop the tunnel. Reason: %{public}s", "\(String(reflecting: reason))")

            self.startedTunnel = false
            self.stopKeyRotation()

            if let device = self.wireguardDevice {
                self.wireguardDevice = nil

                // ignore errors at this point
                return device.stop()
                    .replaceError(with: ())
                    .assertNoFailure()
                    .eraseToAnyPublisher()
            } else {
                return Just(())
                    .eraseToAnyPublisher()
            }
        }.eraseToAnyPublisher()
    }

    private func reloadTunnel() -> AnyPublisher<(), PacketTunnelProviderError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            () -> AnyPublisher<(), PacketTunnelProviderError> in
            guard self.startedTunnel else {
                os_log(.default, log: tunnelProviderLog,
                       "Ignore reloading tunnel settings. The tunnel has not started yet.")

                return Result.Publisher(()).eraseToAnyPublisher()
            }

            guard let wireguardDevice = self.wireguardDevice else {
                os_log(.default, log: tunnelProviderLog,
                       "Ignore reloading tunnel settings. The WireguardDevice is not set yet.")

                return Result.Publisher(()).eraseToAnyPublisher()
            }

            os_log(.default, log: tunnelProviderLog, "Reload tunnel settings")

            return self.makePacketTunnelConfigAndApplyNetworkSettings()
                .flatMap { (packetTunnelConfig) in
                    wireguardDevice
                        .setConfig(configuration: packetTunnelConfig.wireguardConfig)
                        .mapError { PacketTunnelProviderError.updateWireguardConfiguration($0) }
            }
            .receive(on: self.executionQueue)
            .handleEvents(receiveSubscription: { _ in
                // Tell the system that the tunnel is about to reconnect with the new endpoint
                self.reasserting = true
            }, receiveCompletion: { (completion) in
                switch completion {
                case .finished:
                    os_log(.default, log: tunnelProviderLog, "Reloaded the tunnel with new settings")

                case .failure(let error):
                    os_log(.default, log: tunnelProviderLog,
                           "Failed to reload the tunnel with new settings: %{public}s",
                           error.localizedDescription)
                }

                // Tell the system that the tunnel has finished reconnecting
                self.reasserting = false
            }, receiveCancel: {
                // Tell the system that the tunnel has finished reconnecting
                // in the event of task cancellation
                self.reasserting = false
            }).eraseToAnyPublisher()
        }.eraseToAnyPublisher()
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

    /// Make and return `PacketTunnelConfig` after applying network settings and setting the
    /// tunnel connection info
    private func makePacketTunnelConfigAndApplyNetworkSettings()
        -> AnyPublisher<PacketTunnelConfiguration, PacketTunnelProviderError> {
            self.makePacketTunnelConfig()
                .receive(on: executionQueue)
                .flatMap { (packetTunnelConfig) -> AnyPublisher<PacketTunnelConfiguration, PacketTunnelProviderError> in
                    self.setTunnelConnectionInfo(selectorResult: packetTunnelConfig.selectorResult)

                    return self.applyNetworkSettings(packetTunnelConfig: packetTunnelConfig)
                        .map { packetTunnelConfig }
                        .eraseToAnyPublisher()
            }.eraseToAnyPublisher()
    }

    /// Returns a `PacketTunnelConfig` that contains the tunnel configuration and selected relay
    private func makePacketTunnelConfig() -> AnyPublisher<PacketTunnelConfiguration, PacketTunnelProviderError> {
        return getConfigurationPersistentKeychainReference()
            .publisher
            .flatMap { (persistentKeychainReference) in
                Self.readTunnelConfiguration(keychainReference: persistentKeychainReference)
                    .publisher
                    .flatMap { (tunnelConfiguration) in
                        Self.selectRelayEndpoint(relayConstraints: tunnelConfiguration.relayConstraints)
                            .map { (selectorResult) -> PacketTunnelConfiguration in
                                PacketTunnelConfiguration(
                                    persistentKeychainReference: persistentKeychainReference,
                                    tunnelConfig: tunnelConfiguration,
                                    selectorResult: selectorResult)
                        }
                }
        }.eraseToAnyPublisher()
    }

    /// Set system network settings using `PacketTunnelConfig`
    private func applyNetworkSettings(packetTunnelConfig: PacketTunnelConfiguration) -> AnyPublisher<(), PacketTunnelProviderError> {
        let settingsGenerator = PacketTunnelSettingsGenerator(
            mullvadEndpoint: packetTunnelConfig.selectorResult.endpoint,
            tunnelConfiguration: packetTunnelConfig.tunnelConfig
        )

        os_log(.default, log: tunnelProviderLog, "Set tunnel network settings")

        return self.setTunnelNetworkSettings(settingsGenerator.networkSettings())
            .mapError { (error) in
                os_log(.error, log: tunnelProviderLog, "Cannot set network settings: %{public}s", error.localizedDescription)

                return PacketTunnelProviderError.setNetworkSettings(error)
        }
        .receive(on: self.executionQueue)
        .eraseToAnyPublisher()
    }

    /// Returns the persistent keychain reference for the VPN configuration or an error if it's
    /// missing
    private func getConfigurationPersistentKeychainReference() -> Result<Data, PacketTunnelProviderError> {
        return protocolConfiguration.passwordReference.map { .success($0) }
            ?? .failure(.missingKeychainConfigurationReference)
    }

    private func startKeyRotation(persistentKeychainReference: Data) {
        let keyRotationManager = AutomaticKeyRotationManager(
            persistentKeychainReference: persistentKeychainReference
        )

        keyRotationManager.eventHandler = { (keyRotationEvent) in
            self.reloadTunnel().autoDisposableSink(
                cancellableSet: self.cancellableSet,
                receiveCompletion: { (completion) in
                    // no-op
            })
        }

        stopKeyRotation()
        self.keyRotationManager = keyRotationManager

        keyRotationManager.startAutomaticRotation()
    }


    private func stopKeyRotation() {
        keyRotationManager?.stopAutomaticRotation()
        keyRotationManager = nil
    }

    /// Read tunnel configuration from Keychain
    private class func readTunnelConfiguration(keychainReference: Data) -> Result<TunnelConfiguration, PacketTunnelProviderError> {
        TunnelConfigurationManager.load(searchTerm: .persistentReference(keychainReference))
            .mapError { PacketTunnelProviderError.cannotReadTunnelConfiguration($0) }
            .map { $0.tunnelConfiguration }
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private class func selectRelayEndpoint(relayConstraints: RelayConstraints) -> AnyPublisher<RelaySelectorResult, PacketTunnelProviderError> {
        return RelaySelector.loadedFromRelayCache()
            .mapError { PacketTunnelProviderError.readRelayCache($0) }
            .flatMap { (relaySelector) -> Result<RelaySelectorResult, PacketTunnelProviderError>.Publisher in
                return relaySelector
                    .evaluate(with: relayConstraints)
                    .flatMap { .init($0) } ?? .init(.noRelaySatisfyingConstraint)
        }.eraseToAnyPublisher()
    }

    private class func startWireguard(packetFlow: NEPacketTunnelFlow, configuration: WireguardConfiguration) -> AnyPublisher<WireguardDevice, PacketTunnelProviderError> {
        WireguardDevice.fromPacketFlow(packetFlow)
            .publisher
            .flatMap { (device) -> AnyPublisher<WireguardDevice, WireguardDevice.Error> in
                os_log(.default, log: tunnelProviderLog,
                       "Tunnel interface is %{public}s",
                       device.getInterfaceName() ?? "unknown")

                return device.start(configuration: configuration)
                    .map { device }
                    .eraseToAnyPublisher()
        }
        .mapError { PacketTunnelProviderError.startWireguardDevice($0) }
        .eraseToAnyPublisher()
    }
}

extension NETunnelProvider {

    func setTunnelNetworkSettings(_ tunnelNetworkSettings: NETunnelNetworkSettings?) -> Future<(), Error> {
        return Future { (fulfill) in
            self.setTunnelNetworkSettings(tunnelNetworkSettings) { (error) in
                if let error = error {
                    fulfill(.failure(error))
                } else {
                    fulfill(.success(()))
                }
            }
        }
    }

}
