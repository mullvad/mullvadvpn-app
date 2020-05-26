//
//  TunnelManager.swift
//  MullvadVPN
//
//  Created by pronebird on 25/09/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import NetworkExtension
import os

/// An error emitted by all public methods of TunnelManager
enum TunnelManagerError: Error {
    /// Account token is not set
    case missingAccount

    /// A failure to start the tunnel
    case startTunnel(StartTunnelError)

    /// A failure to stop the tunnel
    case stopTunnel(Error)

    /// A failure to load the account with the assumption that it was already set up earlier
    case loadTunnel(LoadTunnelError)

    /// A failure to set the account
    case setAccount(SetAccountError)

    /// A failure to unset the account
    case unsetAccount(UnsetAccountError)

    /// A failure to set the relay constraints
    case setRelayConstraints(TunnelConfigurationManager.Error)

    /// A failure to get the relay constraints
    case getRelayConstraints(TunnelConfigurationManager.Error)

    /// A failure to get a public key used for Wireguard
    case getWireguardPublicKey(TunnelConfigurationManager.Error)

    /// A failure to re-generate a private key used for Wireguard
    case regenerateWireguardPrivateKey(RegenerateWireguardPrivateKeyError)
}

extension TunnelManagerError: LocalizedError {

    var errorDescription: String? {
        switch self {
        case .regenerateWireguardPrivateKey:
            return NSLocalizedString("Cannot regenerate the private key", comment: "")

        case .setAccount:
            return NSLocalizedString("Cannot set up the tunnel", comment: "")

        case .getWireguardPublicKey:
            return NSLocalizedString("Cannot retrieve the public key", comment: "")

        case .startTunnel:
            return NSLocalizedString("Cannot start the tunnel", comment: "")

        case .stopTunnel:
            return NSLocalizedString("Cannot stop the tunnel", comment: "")

        default:
            return nil
        }
    }

    var failureReason: String? {
        switch self {

        case .setAccount(.pushWireguardKey(let pushError)),
             .regenerateWireguardPrivateKey(.replaceWireguardKey(let pushError)):
            switch pushError {
            case .network(let urlError):
                return urlError.localizedDescription

            case .server(let serverError):
                return serverError.errorDescription

            default:
                return NSLocalizedString("Internal error", comment: "")
            }

        default:
            return nil
        }
    }

    var recoverySuggestion: String? {
        switch self {
        case .regenerateWireguardPrivateKey(.replaceWireguardKey(let pushError)):
            switch pushError {
            case .server(let serverError) where serverError.code == .tooManyWireguardKeys:
                return NSLocalizedString("Remove unused WireGuard keys and try again.", comment: "")

            default:
                return nil
            }

        default:
            return nil
        }
    }

}

enum TunnelIpcRequestError: Error {
    /// IPC is not set yet
    case missingIpc

    /// A failure to submit or handle the IPC request
    case send(PacketTunnelIpcError)

    var localizedDescription: String {
        switch self {
        case .missingIpc:
            return "IPC is not initialized yet"

        case .send(let ipcError):
            return "Failure to send an IPC request: \(ipcError.localizedDescription)"
        }
    }
}

enum SetAccountError: Error {
    /// A failure to make the tunnel configuration
    case makeTunnelConfiguration(TunnelConfigurationManager.Error)

    /// A failure to update the tunnel configuration
    case updateTunnelConfiguration(TunnelConfigurationManager.Error)

    /// A failure to push the wireguard key
    case pushWireguardKey(MullvadRpc.Error)

    /// A failure to set up a tunnel
    case setup(SetupTunnelError)
}

enum UnsetAccountError: Error {
    /// A failure to remove the system tunnel
    case removeTunnel(Error)

    /// A failure to remove a tunnel configuration from Keychain
    case removeTunnelConfiguration(TunnelConfigurationManager.Error)
}

enum RegenerateWireguardPrivateKeyError: Error {
    /// A failure to read the public Wireguard key from Keychain
    case readPublicWireguardKey(TunnelConfigurationManager.Error)

    /// A failure to replace the public Wireguard key
    case replaceWireguardKey(MullvadRpc.Error)

    /// A failure to update tunnel configuration
    case updateTunnelConfiguration(TunnelConfigurationManager.Error)
}

enum StartTunnelError: Error {
    /// An error that happened during the tunnel setup stage
    case setup(SetupTunnelError)

    /// System call error
    case system(Error)
}

enum SetupTunnelError: Error {
    /// A failure to load a list of tunnels associated with the app
    case loadTunnels(Error)

    /// A failure to save tunnel preferences
    case saveTunnel(Error)

    /// A failure to reload the tunnel preferences
    case reloadTunnel(Error)

    /// Unable to obtain the keychain reference for the configuration
    case obtainKeychainRef(TunnelConfigurationManager.Error)
}

enum LoadTunnelError: Error {
    /// A failure to load a list of tunnels associated with the app
    case loadTunnels(Error)

    /// A failure to perform a recovery (by removing the tunnel) when the inconsistency between
    /// the given account token and the username saved in the tunnel provide configuration is
    /// detected.
    case removeInconsistentTunnel(Error)
}

enum MapConnectionStatusError: Error {
    /// A failure to send a subsequent IPC request to collect more information, such as tunnel
    /// connection info.
    case ipcRequest(TunnelIpcRequestError)

    /// A failure to map the status because the unknown variant of `NEVPNStatus` was given.
    case unknownStatus(NEVPNStatus)

    /// A failure to map the status because the `NEVPNStatus.invalid` variant was given
    /// This happens when attempting to start a tunnel with configuration that does not exist
    /// anymore in system preferences.
    case invalidConfiguration(NEVPNStatus)
}

/// A enum that describes the tunnel state
enum TunnelState: Equatable {
    /// Connecting the tunnel
    case connecting

    /// Connected the tunnel
    case connected(TunnelConnectionInfo)

    /// Disconnecting the tunnel
    case disconnecting

    /// Disconnected the tunnel
    case disconnected

    /// Reconnecting the tunnel. Normally this state appears in response to changing the
    /// relay constraints and asking the running tunnel to reload the configuration.
    case reconnecting(TunnelConnectionInfo)
}

extension TunnelState: CustomStringConvertible {
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
}

extension TunnelState: CustomDebugStringConvertible {
    var debugDescription: String {
        var output = "TunnelState."

        switch self {
        case .connecting:
            output.append("connecting")

        case .connected(let connectionInfo):
            output.append("connected(")
            output.append(String(reflecting: connectionInfo))
            output.append(")")

        case .disconnecting:
            output.append("disconnecting")

        case .disconnected:
            output.append("disconnected")

        case .reconnecting(let connectionInfo):
            output.append("reconnecting(")
            output.append(String(reflecting: connectionInfo))
            output.append(")")
        }

        return output
    }
}

/// A class that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
class TunnelManager {

    // Switch to stabs on simulator
    #if targetEnvironment(simulator)
    typealias TunnelProviderManagerType = SimulatorTunnelProviderManager
    #else
    typealias TunnelProviderManagerType = NETunnelProviderManager
    #endif

    static let shared = TunnelManager()

    // MARK: - Internal variables

    /// A queue used for dispatching tunnel related jobs that require mutual exclusion
    private let exclusivityQueue = DispatchQueue(label: "net.mullvad.vpn.tunnel-manager.exclusivity-queue")

    /// A queue used for access synchronization to the TunnelManager members
    private let executionQueue = DispatchQueue(label: "net.mullvad.vpn.tunnel-manager.execution-queue")

    private let rpc = MullvadRpc()
    private var tunnelProvider: TunnelProviderManagerType?
    private var tunnelIpc: PacketTunnelIpc?

    /// A subscriber used for tunnel connection status changes
    private var tunnelStatusSubscriber: AnyCancellable?

    /// A subscriber used for mapping a connection status (`NEVPNStatus`) to `TunnelState`
    private var mapTunnelStateSubscriber: AnyCancellable?

    /// An account token associated with the active tunnel
    private var accountToken: String?

    private init() {}

    // MARK: - Public

    @Published private(set) var tunnelState = TunnelState.disconnected

    /// Initialize the TunnelManager with the tunnel from the system
    ///
    /// The given account token is used to ensure that the system tunnel was configured for the same
    /// account. The system tunnel is removed in case of inconsistency.
    func loadTunnel(accountToken: String?) -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            TunnelProviderManagerType.loadAllFromPreferences()
                .mapError { LoadTunnelError.loadTunnels($0) }
                .receive(on: self.executionQueue)
                .flatMap { (tunnels) -> AnyPublisher<(), LoadTunnelError> in

                    // Migrate tunnel configuration if needed
                    if let accountToken = accountToken {
                        self.migrateTunnelConfiguration(accountToken: accountToken)
                    }

                    // No tunnels found. Save the account token.
                    guard let tunnelProvider = tunnels?.first else {
                        self.accountToken = accountToken

                        return Result.Publisher(()).eraseToAnyPublisher()
                    }

                    // Ensure the consistency between the given account token and the one
                    // saved in the system tunnel configuration.
                    if let username = tunnelProvider.protocolConfiguration?.username,
                        let accountToken = accountToken, accountToken == username {
                        self.accountToken = accountToken
                        self.setTunnelProvider(tunnelProvider: tunnelProvider)

                        return Result.Publisher(()).eraseToAnyPublisher()
                    } else {
                        // In case of inconsistency, remove the tunnel
                        return tunnelProvider.removeFromPreferences()
                            .receive(on: self.executionQueue)
                            .mapError { LoadTunnelError.removeInconsistentTunnel($0) }
                            .handleEvents(receiveCompletion: { completion in
                                if case .finished = completion {
                                    self.accountToken = accountToken
                                }
                            }).eraseToAnyPublisher()
                    }
            }.mapError { TunnelManagerError.loadTunnel($0) }
        }.eraseToAnyPublisher()
    }

    /// Refresh tunnel state.
    /// Use this method to update the tunnel state when app transitions from suspended to active
    /// state.
    func refreshTunnelState() -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            () -> AnyPublisher<(), TunnelManagerError> in
            if let status = self.tunnelProvider?.connection.status {
                self.updateTunnelState(connectionStatus: status)
            }
            return Result.Publisher(()).eraseToAnyPublisher()
        }.eraseToAnyPublisher()
    }

    func startTunnel() -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            Just(self.accountToken)
                .setFailureType(to: TunnelManagerError.self)
                .replaceNil(with: .missingAccount)
                .flatMap { (accountToken) in
                    self.setupTunnel(accountToken: accountToken)
                        .mapError { StartTunnelError.setup($0) }
                        .flatMap({ (tunnelProvider) -> Result<(), StartTunnelError>.Publisher in
                            Just(tunnelProvider)
                                .tryMap { try $0.connection.startVPNTunnel() }
                                .mapError { StartTunnelError.system($0) }
                        }).mapError { TunnelManagerError.startTunnel($0) }
            }
            }.eraseToAnyPublisher()
    }

    func stopTunnel() -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) { () -> AnyPublisher<(), TunnelManagerError> in
            if let tunnelProvider = self.tunnelProvider {
                // Disable on-demand when turning off the tunnel to prevent the tunnel from coming
                // back up
                tunnelProvider.isOnDemandEnabled = false

                return tunnelProvider.saveToPreferences()
                    .mapError { TunnelManagerError.stopTunnel($0) }
                    .map { _ -> () in
                        tunnelProvider.connection.stopVPNTunnel()
                        return ()
                }.eraseToAnyPublisher()
            } else {
                return Result.Publisher(()).eraseToAnyPublisher()
            }
        }.eraseToAnyPublisher()
    }

    func setAccount(accountToken: String) -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            self.makeTunnelConfiguration(accountToken: accountToken).publisher
                .mapError { .makeTunnelConfiguration($0) }
                .flatMap { (tunnelConfig: TunnelConfiguration) -> AnyPublisher<(), SetAccountError> in

                    let setupTunnelPublisher = Deferred {
                        self.setupTunnel(accountToken: accountToken)
                            .handleEvents(receiveCompletion: { (completion) in
                                if case .finished = completion {
                                    self.accountToken = accountToken
                                }
                            })
                            .map { _ in () }
                            .mapError { SetAccountError.setup($0) }
                    }

                    // Make sure to avoid pushing the wireguard keys when addresses are assigned
                    guard tunnelConfig.interface.addresses.isEmpty else {
                        return setupTunnelPublisher.eraseToAnyPublisher()
                    }

                    // Send wireguard key to the server
                    let publicKey = tunnelConfig.interface.privateKey.publicKey.rawRepresentation

                    return self.rpc.pushWireguardKey(accountToken: accountToken, publicKey: publicKey)
                        .mapError { SetAccountError.pushWireguardKey($0) }
                        .flatMap { (addresses) in
                            self.updateAssociatedAddresses(
                                accountToken: accountToken,
                                addresses: addresses
                            ).mapError { SetAccountError.updateTunnelConfiguration($0) }
                                .publisher
                                .flatMap { _ in setupTunnelPublisher }
                    }.eraseToAnyPublisher()
            }.mapError { TunnelManagerError.setAccount($0) }
        }.eraseToAnyPublisher()
    }

    /// Remove the account token and remove the active tunnel
    func unsetAccount() -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            Just(self.accountToken)
                .setFailureType(to: TunnelManagerError.self)
                .replaceNil(with: .missingAccount)
                .map { ($0, self.tunnelProvider) }
                .flatMap { (accountToken, tunnelProvider) -> AnyPublisher<(), TunnelManagerError> in

                    let removeKeychainConfigPublisher = Deferred {
                        () -> AnyPublisher<(), UnsetAccountError> in
                        // Load existing configuration
                        switch TunnelConfigurationManager.load(searchTerm: .accountToken(accountToken)) {
                        case .success(let keychainEntry):
                            let publicKey = keychainEntry.tunnelConfiguration
                                .interface
                                .privateKey
                                .publicKey
                                .rawRepresentation

                            // Remove configuration from Keychain
                            return TunnelConfigurationManager.remove(searchTerm: .accountToken(accountToken))
                                .mapError { UnsetAccountError.removeTunnelConfiguration($0) }
                                .publisher
                                .flatMap {
                                    // Remove WireGuard key from master
                                    self.rpc.removeWireguardKey(
                                        accountToken: accountToken,
                                        publicKey: publicKey
                                    )
                                        .retry(1)
                                        .map({ (isRemoved) -> () in
                                            os_log(.debug, "Removed the WireGuard key from server: %{public}s", "\(isRemoved)")
                                            return ()
                                        }).catch({ (error) -> Result<(), UnsetAccountError>.Publisher in
                                            os_log(.error, "Failed to remove the Wireguard key from server: %{public}s", error.localizedDescription)

                                            // Suppress network errors
                                            return Result.Publisher(())
                                        })
                            }.eraseToAnyPublisher()

                        case .failure(let error):
                            // Ignore Keychain errors because that normally means that the Keychain
                            // configuration was already removed and we shouldn't be blocking the
                            // user from logging out
                            os_log(.error, "Failed to read the tunnel configuration from Keychain: %{public}s", error.localizedDescription)

                            return Just(())
                                .setFailureType(to: UnsetAccountError.self)
                                .eraseToAnyPublisher()
                        }
                    }

                    let removeTunnelPublisher = Deferred {
                        () -> AnyPublisher<(), UnsetAccountError> in
                        if let tunnelProvider = tunnelProvider {
                            return tunnelProvider.removeFromPreferences()
                                .catch { (error) -> Result<(), UnsetAccountError>.Publisher in
                                    // Ignore error if the tunnel was already removed by user
                                    if case NEVPNError.configurationInvalid = error {
                                        return .init(())
                                    } else {
                                        return .init(.failure(.removeTunnel(error)))
                                    }
                            }.eraseToAnyPublisher()
                        } else {
                            return Result.Publisher(())
                                .eraseToAnyPublisher()
                        }
                    }

                    return removeTunnelPublisher
                        .receive(on: self.executionQueue)
                        .flatMap { removeKeychainConfigPublisher }
                        .mapError { TunnelManagerError.unsetAccount($0) }
                        .eraseToAnyPublisher()
            }
            .receive(on: self.executionQueue)
            .handleEvents(receiveCompletion: { (completion) in
                if case .finished = completion {
                    self.accountToken = nil
                    self.tunnelProvider = nil
                    self.tunnelIpc = nil

                    self.tunnelStatusSubscriber?.cancel()
                    self.tunnelStatusSubscriber = nil

                    self.mapTunnelStateSubscriber?.cancel()
                    self.mapTunnelStateSubscriber = nil

                    self.tunnelState = .disconnected
                }
            })
        }.eraseToAnyPublisher()
    }

    func regeneratePrivateKey() -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            Just(self.accountToken)
                .setFailureType(to: TunnelManagerError.self)
                .replaceNil(with: .missingAccount)
                .flatMap { (accountToken) -> AnyPublisher<(), TunnelManagerError> in
                    let newPrivateKey = WireguardPrivateKey()

                    return TunnelConfigurationManager.load(searchTerm: .accountToken(accountToken))
                        .map { $0.tunnelConfiguration.interface.privateKey.publicKey }
                        .mapError { RegenerateWireguardPrivateKeyError.readPublicWireguardKey($0) }
                        .publisher
                        .flatMap { (oldPublicKey) in
                            self.rpc.replaceWireguardKey(
                                accountToken: accountToken,
                                oldPublicKey: oldPublicKey.rawRepresentation,
                                newPublicKey: newPrivateKey.publicKey.rawRepresentation)
                                .mapError { RegenerateWireguardPrivateKeyError.replaceWireguardKey($0) }
                                .receive(on: self.executionQueue)
                                .flatMap { (addresses) in
                                    TunnelConfigurationManager.update(searchTerm: .accountToken(accountToken)) {
                                        (tunnelConfiguration) in
                                        tunnelConfiguration.interface.privateKey = newPrivateKey
                                        tunnelConfiguration.interface.addresses = [
                                            addresses.ipv4Address,
                                            addresses.ipv6Address
                                        ]
                                    }
                                    .mapError { .updateTunnelConfiguration($0) }
                                    .map { _ in () }
                                    .publisher
                            }.receive(on: self.executionQueue)
                                .flatMap { _ in
                                    // Ignore Packet Tunnel IPC errors but log them
                                    self.reloadPacketTunnelConfiguration()
                                        .handleEvents(receiveCompletion: { (completion) in
                                            if case .failure(let error) = completion {
                                                os_log(.error, "Failed to tell the tunnel to reload configuration: %{public}s", error.localizedDescription)
                                            }
                                        })
                                        .replaceError(with: ())
                                        .setFailureType(to: RegenerateWireguardPrivateKeyError.self)
                            }
                    }
                    .mapError { TunnelManagerError.regenerateWireguardPrivateKey($0) }
                    .eraseToAnyPublisher()
            }
        }.eraseToAnyPublisher()
    }

    func setRelayConstraints(_ constraints: RelayConstraints) -> AnyPublisher<(), TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            Just(self.accountToken)
                .setFailureType(to: TunnelManagerError.self)
                .replaceNil(with: .missingAccount)
                .flatMap { (accountToken) in
                    TunnelConfigurationManager
                        .update(searchTerm: .accountToken(accountToken)) { (tunnelConfig) in
                            tunnelConfig.relayConstraints = constraints
                    }.mapError { TunnelManagerError.setRelayConstraints($0) }
                        .publisher
                        .flatMap { _ in
                            // Ignore Packet Tunnel IPC errors but log them
                            self.reloadPacketTunnelConfiguration()
                                .replaceError(with: ())
                                .setFailureType(to: TunnelManagerError.self)
                                .handleEvents(receiveCompletion: { (completion) in
                                    if case .failure(let error) = completion {
                                        os_log(.error, "Failed to tell the tunnel to reload configuration: %{public}s", error.localizedDescription)
                                    }
                                })
                    }
            }
        }.eraseToAnyPublisher()
    }

    func getRelayConstraints() -> AnyPublisher<RelayConstraints, TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            Just(self.accountToken)
                .setFailureType(to: TunnelManagerError.self)
                .replaceNil(with: .missingAccount)
                .flatMap { (accountToken) in
                    TunnelConfigurationManager.load(searchTerm: .accountToken(accountToken))
                        .map { $0.tunnelConfiguration.relayConstraints }
                        .flatMapError { (error) -> Result<RelayConstraints, TunnelConfigurationManager.Error> in
                            // Return default constraints if the config is not found in Keychain
                            if case .lookupEntry(.itemNotFound) = error {
                                return .success(TunnelConfiguration().relayConstraints)
                            } else {
                                return .failure(error)
                            }
                    }.mapError { .getRelayConstraints($0) }
                        .publisher
            }
        }.eraseToAnyPublisher()
    }

    func getWireguardPublicKey() -> AnyPublisher<WireguardPublicKey, TunnelManagerError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            Just(self.accountToken)
                .setFailureType(to: TunnelManagerError.self)
                .replaceNil(with: .missingAccount)
                .flatMap { (accountToken) in
                    TunnelConfigurationManager.load(searchTerm: .accountToken(accountToken))
                        .map { $0.tunnelConfiguration.interface.privateKey.publicKey }
                        .mapError { .getWireguardPublicKey($0) }
                        .publisher
            }
        }.eraseToAnyPublisher()
    }

    // MARK: - Private

    /// Tell Packet Tunnel process to reload the tunnel configuration
    private func reloadPacketTunnelConfiguration() -> AnyPublisher<(), TunnelIpcRequestError> {
        return executeIpcRequest { $0.reloadConfiguration() }
    }

    /// Ask Packet Tunnel process to return the current tunnel connection info
    private func getTunnelConnectionInfo() -> AnyPublisher<TunnelConnectionInfo, TunnelIpcRequestError> {
        return executeIpcRequest { $0.getTunnelInformation() }
    }

    /// IPC interactor helper that automatically maps the `PacketTunnelIpcError` to
    /// `TunnelIpcRequestError`
    private func executeIpcRequest<T>(
        _ body: @escaping (PacketTunnelIpc) -> AnyPublisher<T, PacketTunnelIpcError>
    ) -> AnyPublisher<T, TunnelIpcRequestError>
    {
        Just(tunnelIpc)
            .setFailureType(to: TunnelIpcRequestError.self)
            .replaceNil(with: .missingIpc)
            .flatMap { (tunnelIpc) in
                body(tunnelIpc)
                    .mapError { .send($0) }
        }.eraseToAnyPublisher()
    }

    /// Set the instance of the active tunnel and add the tunnel status observer
    private func setTunnelProvider(tunnelProvider: TunnelProviderManagerType) {
        guard self.tunnelProvider != tunnelProvider else { return }

        let connection = tunnelProvider.connection

        // Save the new active tunnel provider
        self.tunnelProvider = tunnelProvider

        // Set up tunnel IPC
        if let session = connection as? VPNTunnelProviderSessionProtocol {
            self.tunnelIpc = PacketTunnelIpc(session: session)
        }

        // Register for tunnel connection status changes
        tunnelStatusSubscriber = NotificationCenter.default.publisher(for: .NEVPNStatusDidChange, object: connection)
            .map { notification in
                (notification.object as? VPNConnectionProtocol)?.status
        }
        .receive(on: executionQueue)
        .sink { [weak self] (connectionStatus) in
            guard let connectionStatus = connectionStatus else { return }

            self?.updateTunnelState(connectionStatus: connectionStatus)
        }

        // Update the existing connection status
        updateTunnelState(connectionStatus: connection.status)
    }

    /// Initiates the `tunnelState` update
    private func updateTunnelState(connectionStatus: NEVPNStatus) {
        os_log(.default, "VPN Status: %{public}s", "\(connectionStatus)")

        mapTunnelStateSubscriber = mapTunnelState(connectionStatus: connectionStatus)
            .receive(on: executionQueue)
            .sink(receiveCompletion: { (completion) in
                if case .failure(let error) = completion {
                    os_log(.error, "Failed to map the tunnel state: %{public}s", error.localizedDescription)
                }
            }, receiveValue: { (tunnelState) in
                os_log(.default, "Set tunnel state: %{public}s", String(reflecting: tunnelState))
                self.tunnelState = tunnelState
            })
    }

    /// Maps `NEVPNStatus` to `TunnelState`.
    /// Collects the `TunnelConnectionInfo` from the tunnel via IPC if needed before assigning the
    /// `tunnelState`
    private func mapTunnelState(connectionStatus: NEVPNStatus) -> AnyPublisher<TunnelState, MapConnectionStatusError> {
        Just(connectionStatus)
            .setFailureType(to: MapConnectionStatusError.self)
            .flatMap { (connectionStatus) -> AnyPublisher<TunnelState, MapConnectionStatusError> in
                switch connectionStatus {
                case .connected:
                    return self.getTunnelConnectionInfo()
                        .mapError { .ipcRequest($0) }
                        .map { .connected($0) }
                        .eraseToAnyPublisher()

                case .connecting:
                    return Result.Publisher(TunnelState.connecting)
                        .eraseToAnyPublisher()

                case .disconnected:
                    return Result.Publisher(TunnelState.disconnected)
                        .eraseToAnyPublisher()

                case .disconnecting:
                    return Result.Publisher(TunnelState.disconnecting)
                        .eraseToAnyPublisher()

                case .reasserting:
                    return self.getTunnelConnectionInfo()
                        .mapError { .ipcRequest($0) }
                        .map { .reconnecting($0) }
                        .eraseToAnyPublisher()

                case .invalid:
                    return Fail(error: MapConnectionStatusError.invalidConfiguration(connectionStatus))
                        .eraseToAnyPublisher()

                @unknown default:
                    return Fail(error: MapConnectionStatusError.unknownStatus(connectionStatus))
                        .eraseToAnyPublisher()
                }
        }.eraseToAnyPublisher()
    }

    /// Retrieve the existing TunnelConfiguration or create a new one
    private func makeTunnelConfiguration(accountToken: String) -> Result<TunnelConfiguration, TunnelConfigurationManager.Error> {
        TunnelConfigurationManager.load(searchTerm: .accountToken(accountToken))
            .map { $0.tunnelConfiguration }
            .flatMapError { (error) -> Result<TunnelConfiguration, TunnelConfigurationManager.Error> in
                // Return default tunnel configuration if the config is not found in Keychain
                if case .lookupEntry(.itemNotFound) = error {
                    let defaultConfiguration = TunnelConfiguration()

                    return TunnelConfigurationManager
                        .add(configuration: defaultConfiguration, account: accountToken)
                        .map { defaultConfiguration }
                } else {
                    return .failure(error)
                }
        }
    }

    private func setupTunnel(accountToken: String) -> AnyPublisher<TunnelProviderManagerType, SetupTunnelError> {
        TunnelProviderManagerType.loadAllFromPreferences()
            .receive(on: executionQueue)
            .mapError { SetupTunnelError.loadTunnels($0) }
            .map { (tunnels) in
                // Return the first available tunnel or make a new one
                return tunnels?.first ?? TunnelProviderManagerType()
        }
        .flatMap { (tunnelProvider) in
            TunnelConfigurationManager.getPersistentKeychainReference(account: accountToken)
                .mapError { SetupTunnelError.obtainKeychainRef($0) }
                .map { (tunnelProvider, $0) }
                .publisher
        }
        .flatMap { (tunnelProvider, passwordReference) -> AnyPublisher<TunnelProviderManagerType, SetupTunnelError> in
            tunnelProvider.isEnabled = true
            tunnelProvider.localizedDescription = "WireGuard"
            tunnelProvider.protocolConfiguration = self.makeProtocolConfiguration(
                accountToken: accountToken,
                passwordReference: passwordReference
            )

            // Enable on-demand VPN, always connect the tunnel when on Wi-Fi or cellular
            let alwaysOnRule = NEOnDemandRuleConnect()
            alwaysOnRule.interfaceTypeMatch = .any
            tunnelProvider.onDemandRules = [alwaysOnRule]
            tunnelProvider.isOnDemandEnabled = true

            return tunnelProvider.saveToPreferences()
                .mapError { SetupTunnelError.saveTunnel($0) }
                .flatMap {
                    // Refresh connection status after saving the tunnel preferences.
                    // Basically it's only necessary to do for new instances of
                    // `NETunnelProviderManager`, but we do that for the existing ones too for
                    // simplicity as it has no side effects.
                    tunnelProvider.loadFromPreferences()
                        .mapError { SetupTunnelError.reloadTunnel($0) }
            }
            .map { tunnelProvider }
            .receive(on: self.executionQueue)
            .handleEvents(receiveCompletion: { (completion) in
                if case .finished = completion {
                    self.setTunnelProvider(tunnelProvider: tunnelProvider)
                }
            }).eraseToAnyPublisher()
        }.eraseToAnyPublisher()
    }

    /// Produce the new tunnel provider protocol configuration
    private func makeProtocolConfiguration(accountToken: String, passwordReference: Data) -> NETunnelProviderProtocol {
        let protocolConfig = NETunnelProviderProtocol()
        protocolConfig.providerBundleIdentifier = ApplicationConfiguration.packetTunnelExtensionIdentifier
        protocolConfig.serverAddress = ""
        protocolConfig.username = accountToken
        protocolConfig.passwordReference = passwordReference

        return protocolConfig
    }

    private func updateAssociatedAddresses(
        accountToken: String,
        addresses: WireguardAssociatedAddresses
    ) -> Result<TunnelConfiguration, TunnelConfigurationManager.Error>
    {
        TunnelConfigurationManager.update(searchTerm: .accountToken(accountToken)) { (tunnelConfig) in
            tunnelConfig.interface.addresses = [
                addresses.ipv4Address,
                addresses.ipv6Address
            ]
        }
    }

    private func migrateTunnelConfiguration(accountToken: String) {
        let result = TunnelConfigurationManager
            .migrateKeychainEntry(searchTerm: .accountToken(accountToken))

        switch result {
        case .success(let migrated):
            if migrated {
                os_log("Migrated Keychain tunnel configuration")
            } else {
                os_log("Tunnel configuration is up to date. No migration needed.")
            }

        case .failure(let error):
            os_log("Failed to migrate tunnel configuration: %{public}s",
                   error.localizedDescription)
        }
    }

}

/// Convenience methods to provide `Future` based alternatives for working with
/// `TunnelProviderManager`
private extension VPNTunnelProviderManagerProtocol {

    static func loadAllFromPreferences() -> Future<[SelfType]?, Error> {
        Future { (fulfill) in
            self.loadAllFromPreferences { (tunnels, error) in
                fulfill(error.flatMap { .failure($0) } ?? .success(tunnels))
            }
        }
    }

    func loadFromPreferences() -> Future<(), Error> {
        Future { (fulfill) in
            self.loadFromPreferences { (error) in
                fulfill(error.flatMap { .failure($0) } ?? .success(()))
            }
        }
    }

    func saveToPreferences() -> Future<(), Error> {
        Future { (fulfill) in
            self.saveToPreferences { (error) in
                fulfill(error.flatMap { .failure($0) } ?? .success(()))
            }
        }
    }

    func removeFromPreferences() -> Future<(), Error> {
        Future { (fulfill) in
            self.removeFromPreferences { (error) in
                fulfill(error.flatMap { .failure($0) } ?? .success(()))
            }
        }
    }

}
