//
//  TunnelManager.swift
//  MullvadVPN
//
//  Created by pronebird on 25/09/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import BackgroundTasks
import Foundation
import NetworkExtension
import UIKit
import Logging
import class WireGuardKit.PublicKey

/// A class that provides a convenient interface for VPN tunnels configuration, manipulation and
/// monitoring.
class TunnelManager {
    /// Private key rotation interval (in seconds)
    private static let privateKeyRotationInterval: TimeInterval = 60 * 60 * 24 * 4

    /// Private key rotation retry interval (in seconds)
    private static let privateKeyRotationFailureRetryInterval: TimeInterval = 60 * 15

    /// Operation categories
    private enum OperationCategory {
        static let manageTunnelProvider = "TunnelManager.manageTunnelProvider"
        static let changeTunnelSettings = "TunnelManager.changeTunnelSettings"
        static let notifyTunnelSettingsChange = "TunnelManager.notifyTunnelSettingsChange"
    }

    // Switch to stabs on simulator
    #if targetEnvironment(simulator)
    typealias TunnelProviderManagerType = SimulatorTunnelProviderManager
    #else
    typealias TunnelProviderManagerType = NETunnelProviderManager
    #endif

    static let shared = TunnelManager()

    // MARK: - Internal variables

    private let logger = Logger(label: "TunnelManager")
    private let stateQueue = DispatchQueue(label: "TunnelManager.stateQueue")
    private let operationQueue: OperationQueue = {
        let operationQueue = OperationQueue()
        operationQueue.name = "TunnelManager.operationQueue"
        return operationQueue
    }()

    private var tunnelProvider: TunnelProviderManagerType?
    private var ipcSession: TunnelIPC.Session?
    private var tunnelConnectionInfoToken: PromiseCancellationToken?

    private let stateLock = NSLock()
    private let observerList = ObserverList<AnyTunnelObserver>()

    /// A VPN connection status observer
    private var connectionStatusObserver: NSObjectProtocol?

    private(set) var tunnelInfo: TunnelInfo? {
        set {
            stateLock.withCriticalBlock {
                _tunnelInfo = newValue
                tunnelInfoDidChange(newValue)
            }
        }
        get {
            return stateLock.withCriticalBlock {
                return _tunnelInfo
            }
        }
    }

    private var _tunnelInfo: TunnelInfo?
    private var _tunnelState = TunnelState.disconnected

    private(set) var tunnelState: TunnelState {
        set {
            stateLock.withCriticalBlock {
                guard _tunnelState != newValue else { return }

                logger.info("Set tunnel state: \(newValue)")

                _tunnelState = newValue

                DispatchQueue.main.async {
                    self.observerList.forEach { (observer) in
                        observer.tunnelManager(self, didUpdateTunnelState: newValue)
                    }
                }
            }
        }
        get {
            return stateLock.withCriticalBlock {
                return _tunnelState
            }
        }
    }

    private init() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(applicationDidBecomeActive),
            name: UIApplication.didBecomeActiveNotification,
            object: nil
        )
    }

    // MARK: - Periodic private key rotation

    private var privateKeyRotationTimer: DispatchSourceTimer?
    private var isRunningPeriodicPrivateKeyRotation = false

    func startPeriodicPrivateKeyRotation() {
        stateQueue.async {
            guard !self.isRunningPeriodicPrivateKeyRotation else { return }

            self.logger.debug("Start periodic private key rotation")

            self.isRunningPeriodicPrivateKeyRotation = true

            self.updatePrivateKeyRotationTimer()
        }
    }

    func stopPeriodicPrivateKeyRotation() {
        stateQueue.async {
            guard self.isRunningPeriodicPrivateKeyRotation else { return }

            self.logger.debug("Stop periodic private key rotation")

            self.isRunningPeriodicPrivateKeyRotation = false

            self.privateKeyRotationTimer?.cancel()
            self.privateKeyRotationTimer = nil
        }
    }

    private func updatePrivateKeyRotationTimer() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        guard self.isRunningPeriodicPrivateKeyRotation else { return }

        if let tunnelInfo = self.tunnelInfo {
            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let scheduleDate = Date(timeInterval: Self.privateKeyRotationInterval, since: creationDate)

            schedulePrivateKeyRotationTimer(scheduleDate)
        } else {
            privateKeyRotationTimer?.cancel()
            privateKeyRotationTimer = nil
        }
    }

    /// Schedule new private key rotation timer.
    private func schedulePrivateKeyRotationTimer(_ scheduleDate: Date) {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        var cancellationToken: PromiseCancellationToken?

        let timer = DispatchSource.makeTimerSource(flags: [], queue: self.stateQueue)

        timer.setEventHandler { [weak self] in
            guard let self = self else { return }

            self.rotatePrivateKey()
                .receive(on: self.stateQueue)
                .storeCancellationToken(in: &cancellationToken)
                .observe { completion in
                    guard !completion.isCancelled else { return }

                    if let scheduleDate = self.handlePrivateKeyRotationCompletion(completion: completion) {
                        self.schedulePrivateKeyRotationTimer(scheduleDate)
                    }
                }
        }

        timer.setCancelHandler {
            cancellationToken?.cancel()
        }

        // Cancel active timer
        privateKeyRotationTimer?.cancel()

        // Assign new timer
        privateKeyRotationTimer = timer

        // Schedule and activate
        timer.schedule(wallDeadline: .now() + scheduleDate.timeIntervalSinceNow)
        timer.activate()

        self.logger.debug("Schedule next private key rotation on \(scheduleDate.logFormatDate())")
    }

    // MARK: - Public methods

    /// Initialize the TunnelManager with the tunnel from the system.
    ///
    /// The given account token is used to ensure that the system tunnel was configured for the same
    /// account. The system tunnel is removed in case of inconsistency.
    func loadTunnel(accountToken: String?) -> Result<(), TunnelManager.Error>.Promise {
        return TunnelProviderManagerType.loadAllFromPreferences()
            .receive(on: self.stateQueue)
            .mapError { error in
                return .loadAllVPNConfigurations(error)
            }.mapThen { tunnels in
                return Result.Promise { resolver in
                    self.initializeManager(accountToken: accountToken, tunnels: tunnels) { result in
                        self.updatePrivateKeyRotationTimer()
                        resolver.resolve(value: result)
                    }
                }
            }
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.loadAccount")
    }

    func startTunnel() {
        Result<(), TunnelManager.Error>.Promise { resolver in
            guard let tunnelInfo = self.tunnelInfo else {
                resolver.resolve(value: .failure(.missingAccount))
                return
            }

            switch self.tunnelState {
            case .disconnecting(.nothing):
                self.tunnelState = .disconnecting(.reconnect)
                resolver.resolve(value: .success(()))

            case .disconnected, .pendingReconnect:
                RelayCache.Tracker.shared.read()
                    .mapError { error in
                        return .readRelays(error)
                    }
                    .receive(on: self.stateQueue)
                    .flatMap { cachedRelays in
                        return RelaySelector.evaluate(
                            relays: cachedRelays.relays,
                            constraints: tunnelInfo.tunnelSettings.relayConstraints
                        ).map { .success($0) } ?? .failure(.cannotSatisfyRelayConstraints)
                    }
                    .mapThen { selectorResult in
                        return self.makeTunnelProvider(accountToken: tunnelInfo.token)
                            .receive(on: self.stateQueue)
                            .flatMap { tunnelProvider in
                                self.setTunnelProvider(tunnelProvider: tunnelProvider)

                                var tunnelOptions = PacketTunnelOptions()

                                _ = Result { try tunnelOptions.setSelectorResult(selectorResult) }
                                .mapError { error -> Swift.Error in
                                    self.logger.error(chainedError: AnyChainedError(error), message: "Failed to encode relay selector result.")
                                    return error
                                }

                                self.tunnelState = .connecting(selectorResult.tunnelConnectionInfo)

                                return Result { try tunnelProvider.connection.startVPNTunnel(options: tunnelOptions.rawOptions()) }
                                .mapError { error in
                                    return .startVPNTunnel(error)
                                }
                            }
                    }.observe { completion in
                        resolver.resolve(completion: completion)
                    }

            default:
                // Do not attempt to start the tunnel in all other cases.
                resolver.resolve(value: .success(()))
            }
        }
        .schedule(on: stateQueue)
        .run(on: operationQueue, categories: [OperationCategory.manageTunnelProvider])
        .requestBackgroundTime(taskName: "TunnelManager.startTunnel")
        .onFailure { error in
            self.sendFailureToObservers(error)
        }
        .observe { _ in }
    }

    func stopTunnel() {
        Result<(), Error>.Promise { resolver in
            guard let tunnelProvider = self.tunnelProvider else {
                resolver.resolve(value: .failure(.missingAccount))
                return
            }

            switch self.tunnelState {
            case .disconnecting(.reconnect):
                self.tunnelState = .disconnecting(.nothing)
                resolver.resolve(value: .success(()))

            case .connected, .connecting:
                // Disable on-demand when stopping the tunnel to prevent it from coming back up
                tunnelProvider.isOnDemandEnabled = false

                tunnelProvider.saveToPreferences()
                    .mapError { error in
                        return Error.saveVPNConfiguration(error)
                    }
                    .observe { completion in
                        tunnelProvider.connection.stopVPNTunnel()
                        resolver.resolve(completion: completion)
                    }

            default:
                resolver.resolve(value: .success(()))
            }
        }
        .schedule(on: stateQueue)
        .run(on: operationQueue, categories: [OperationCategory.manageTunnelProvider])
        .requestBackgroundTime(taskName: "TunnelManager.stopTunnel")
        .onFailure { error in
            self.sendFailureToObservers(error)
        }
        .observe { _ in }
    }

    func reconnectTunnel() {
        notifyTunnelOnSettingsChange().observe { _ in }
    }

    func setAccount(accountToken: String) -> Result<(), TunnelManager.Error>.Promise {
        return Promise.deferred { Self.makeTunnelSettings(accountToken: accountToken) }
            .mapThen { tunnelSettings -> Result<TunnelSettings, Error>.Promise in
                let interfaceSettings = tunnelSettings.interface
                guard interfaceSettings.addresses.isEmpty else {
                    return .success(tunnelSettings)
                }

                // Push wireguard key if addresses were not received yet
                return self.pushWireguardKeyAndUpdateSettings(accountToken: accountToken, publicKey: interfaceSettings.publicKey)
            }
            .receive(on: self.stateQueue)
            .onSuccess { tunnelSettings in
                self.tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: tunnelSettings)
                self.updatePrivateKeyRotationTimer()
            }
            .setOutput(())
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.setAccount")
    }

    /// Remove the account token and remove the active tunnel
    func unsetAccount() ->  Result<(), TunnelManager.Error>.Promise {
        return Promise.deferred { self.tunnelInfo }
            .some(or: Error.missingAccount)
            .mapThen { tunnelInfo in
                let publicKey = tunnelInfo.tunnelSettings.interface.publicKey

                return self.removeWireguardKeyFromServer(accountToken: tunnelInfo.token, publicKey: publicKey)
                    .receive(on: self.stateQueue)
                    .then { result -> Result<(), Error>.Promise in
                        switch result {
                        case .success(let isRemoved):
                            self.logger.warning("Removed the WireGuard key from server: \(isRemoved)")

                        case .failure(let error):
                            self.logger.error(chainedError: error, message: "Unset account error")
                        }

                        // Unregister from receiving the tunnel state changes
                        self.unregisterConnectionObserver()
                        self.tunnelConnectionInfoToken = nil
                        self.tunnelState = .disconnected
                        self.ipcSession = nil

                        // Remove settings from Keychain
                        if case .failure(let error) = TunnelSettingsManager.remove(searchTerm: .accountToken(tunnelInfo.token)) {
                            // Ignore Keychain errors because that normally means that the Keychain
                            // configuration was already removed and we shouldn't be blocking the
                            // user from logging out
                            self.logger.error(
                                chainedError: error,
                                message: "Failure to remove tunnel setting from keychain when unsetting user account"
                            )
                        }

                        self.tunnelInfo = nil
                        self.updatePrivateKeyRotationTimer()

                        guard let tunnelProvider = self.tunnelProvider else {
                            return .success(())
                        }

                        self.tunnelProvider = nil

                        // Remove VPN configuration
                        return tunnelProvider.removeFromPreferences()
                            .flatMapError { error -> Result<(), Error> in
                                // Ignore error but log it
                                self.logger.error(
                                    chainedError: Error.removeVPNConfiguration(error),
                                    message: "Failure to remove system VPN configuration when unsetting user account."
                                )

                                return .success(())
                            }
                    }
            }
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.manageTunnelProvider, OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.unsetAccount")
    }

    func regeneratePrivateKey() -> Result<(), TunnelManager.Error>.Promise {
        return Promise.deferred { self.tunnelInfo }
            .some(or: .missingAccount)
            .mapThen { tunnelInfo in
                let newPrivateKey = PrivateKeyWithMetadata()
                let oldPublicKeyMetadata = tunnelInfo.tunnelSettings.interface
                    .privateKey
                    .publicKeyWithMetadata

                return self.replaceWireguardKeyAndUpdateSettings(
                    accountToken: tunnelInfo.token,
                    oldPublicKey: oldPublicKeyMetadata,
                    newPrivateKey: newPrivateKey
                ).onSuccess { newTunnelSettings in
                    self.tunnelInfo?.tunnelSettings = newTunnelSettings
                    self.updatePrivateKeyRotationTimer()

                    self.notifyTunnelOnSettingsChange().observe { _ in }
                }
                .setOutput(())
            }
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.regeneratePrivateKey")
    }

    func rotatePrivateKey() -> Result<KeyRotationResult, TunnelManager.Error>.Promise {
        return Promise.deferred { self.tunnelInfo }
            .some(or: .missingAccount)
            .mapThen { tunnelInfo in
                let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
                let timeInterval = Date().timeIntervalSince(creationDate)

                guard timeInterval >= Self.privateKeyRotationInterval else {
                    return .success(.throttled(creationDate))
                }

                let newPrivateKey = PrivateKeyWithMetadata()
                let oldPublicKeyMetadata = tunnelInfo.tunnelSettings.interface
                    .privateKey
                    .publicKeyWithMetadata

                return self.replaceWireguardKeyAndUpdateSettings(accountToken: tunnelInfo.token, oldPublicKey: oldPublicKeyMetadata, newPrivateKey: newPrivateKey)
                    .onSuccess { newTunnelSettings in
                        self.tunnelInfo?.tunnelSettings = newTunnelSettings
                    }
                    .mapThen { _ in
                        return self.notifyTunnelOnSettingsChange().then { _ in
                            return .success(.finished)
                        }
                    }
            }
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.rotatePrivateKey")
    }

    func setRelayConstraints(_ newConstraints: RelayConstraints) -> Result<(), TunnelManager.Error>.Promise {
        return Promise.deferred { self.tunnelInfo }
            .some(or: .missingAccount)
            .flatMap { tunnelInfo in
                return Self.updateTunnelSettings(accountToken: tunnelInfo.token) { tunnelSettings in
                    tunnelSettings.relayConstraints = newConstraints
                }
            }
            .onSuccess { newTunnelSettings in
                self.tunnelInfo?.tunnelSettings = newTunnelSettings
                self.notifyTunnelOnSettingsChange().observe { _ in }
            }
            .setOutput(())
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.setRelayConstraints")
    }

    func setDNSSettings(_ newDNSSettings: DNSSettings) -> Result<(), TunnelManager.Error>.Promise {
        return Promise.deferred { self.tunnelInfo }
            .some(or: .missingAccount)
            .flatMap { tunnelInfo in
                return Self.updateTunnelSettings(accountToken: tunnelInfo.token) { tunnelSettings in
                    tunnelSettings.interface.dnsSettings = newDNSSettings
                }
            }
            .onSuccess { newTunnelSettings in
                self.tunnelInfo?.tunnelSettings = newTunnelSettings
                self.notifyTunnelOnSettingsChange().observe { _ in }
            }
            .setOutput(())
            .schedule(on: stateQueue)
            .run(on: operationQueue, categories: [OperationCategory.changeTunnelSettings])
            .requestBackgroundTime(taskName: "TunnelManager.setDNSSettings")
    }

    // MARK: - Tunnel observeration

    /// Add tunnel observer.
    /// In order to cancel the observation, either call `removeTunnelObserver(_:)` or simply release
    /// the observer.
    func addObserver<T: TunnelObserver>(_ observer: T) {
        observerList.append(AnyTunnelObserver(observer))
    }

    /// Remove tunnel observer.
    func removeObserver<T: TunnelObserver>(_ observer: T) {
        observerList.remove(AnyTunnelObserver(observer))
    }

    // MARK: - Private methods

    private func tunnelInfoDidChange(_ newTunnelInfo: TunnelInfo?) {
        // Notify observers
        DispatchQueue.main.async {
            self.observerList.forEach { (observer) in
                observer.tunnelManager(self, didUpdateTunnelSettings: newTunnelInfo)
            }
        }
    }

    private func initializeManager(accountToken: String?, tunnels: [TunnelProviderManagerType]?, completionHandler: @escaping (Result<(), Error>) -> Void) {
        // Migrate the tunnel settings if needed
        let migrationResult = accountToken.map { self.migrateTunnelSettings(accountToken: $0) }
        switch migrationResult {
        case .success, .none:
            break
        case .failure(let migrationError):
            completionHandler(.failure(migrationError))
            return
        }

        switch (tunnels?.first, accountToken) {
        // Case 1: tunnel exists and account token is set.
        // Verify that tunnel can access the configuration via the persistent keychain reference
        // stored in `passwordReference` field of VPN configuration.
        case (.some(let tunnelProvider), .some(let accountToken)):
            let verificationResult = self.verifyTunnel(tunnelProvider: tunnelProvider, expectedAccountToken: accountToken)
            let tunnelSettingsResult = Self.loadTunnelSettings(accountToken: accountToken)

            switch (verificationResult, tunnelSettingsResult) {
            case (.success(true), .success(let keychainEntry)):
                self.tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: keychainEntry.tunnelSettings)
                self.setTunnelProvider(tunnelProvider: tunnelProvider)

                completionHandler(.success(()))

            // Remove the tunnel when failed to verify it but successfuly loaded the tunnel
            // settings.
            case (.failure(let verificationError), .success(let keychainEntry)):
                self.logger.error(chainedError: verificationError, message: "Failed to verify the tunnel but successfully loaded the tunnel settings. Removing the tunnel.")

                // Identical code path as the case below.
                fallthrough

            // Remove the tunnel with corrupt configuration.
            // It will be re-created upon the first attempt to connect the tunnel.
            case (.success(false), .success(let keychainEntry)):
                tunnelProvider.removeFromPreferences()
                    .receive(on: self.stateQueue)
                    .mapError { error in
                        return .removeInconsistentVPNConfiguration(error)
                    }
                    .onSuccess { _ in
                        self.tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: keychainEntry.tunnelSettings)
                    }
                    .observe { completion in
                        completionHandler(completion.unwrappedValue!)
                    }

            // Remove the tunnel when failed to verify the tunnel and load tunnel settings.
            case (.failure(let verificationError), .failure(_)):
                self.logger.error(chainedError: verificationError, message: "Failed to verify the tunnel and load tunnel settings. Removing the tunnel.")

                tunnelProvider.removeFromPreferences()
                    .mapError { error in
                        return .removeInconsistentVPNConfiguration(error)
                    }
                    .flatMap { _ in
                        return .failure(verificationError)
                    }
                    .observe { completion in
                        completionHandler(completion.unwrappedValue!)
                    }

            // Remove the tunnel when the app is not able to read tunnel settings
            case (.success(_), .failure(let settingsReadError)):
                self.logger.error(chainedError: settingsReadError, message: "Failed to load tunnel settings. Removing the tunnel.")

                tunnelProvider.removeFromPreferences()
                    .mapError { error in
                        return .removeInconsistentVPNConfiguration(error)
                    }
                    .flatMap { _ in
                        return .failure(settingsReadError)
                    }
                    .observe { completion in
                        completionHandler(completion.unwrappedValue!)
                    }
            }

        // Case 2: tunnel exists but account token is unset.
        // Remove the orphaned tunnel.
        case (.some(let tunnelProvider), .none):
            tunnelProvider.removeFromPreferences()
                .mapError { error in
                    return .removeInconsistentVPNConfiguration(error)
                }
                .observe { completion in
                    completionHandler(completion.unwrappedValue!)
                }

        // Case 3: tunnel does not exist but the account token is set.
        // Verify that tunnel settings exists in keychain.
        case (.none, .some(let accountToken)):
            switch Self.loadTunnelSettings(accountToken: accountToken) {
            case .success(let keychainEntry):
                self.tunnelInfo = TunnelInfo(token: accountToken, tunnelSettings: keychainEntry.tunnelSettings)

                completionHandler(.success(()))

            case .failure(let error):
                completionHandler(.failure(error))
            }

        // Case 4: no tunnels exist and account token is unset.
        case (.none, .none):
            completionHandler(.success(()))
        }
    }

    private func verifyTunnel(tunnelProvider: TunnelProviderManagerType, expectedAccountToken accountToken: String) -> Result<Bool, Error> {
        // Check that the VPN configuration points to the same account token
        guard let username = tunnelProvider.protocolConfiguration?.username, username == accountToken else {
            logger.warning("The token assigned to the VPN configuration does not match the logged in account.")
            return .success(false)
        }

        // Check that the passwordReference, containing the keychain reference for tunnel
        // configuration, is set.
        guard let keychainReference = tunnelProvider.protocolConfiguration?.passwordReference else {
            logger.warning("VPN configuration is missing the passwordReference.")
            return .success(false)
        }

        // Verify that the keychain reference points to the existing entry in Keychain.
        // Bad reference is possible when migrating the user data from one device to the other.
        return TunnelSettingsManager.exists(searchTerm: .persistentReference(keychainReference))
            .mapError { (error) -> Error in
                logger.error(chainedError: error, message: "Failed to verify the persistent keychain reference for tunnel settings.")

                return Error.readTunnelSettings(error)
            }
    }

    /// Set the instance of the active tunnel and add the tunnel status observer
    private func setTunnelProvider(tunnelProvider: TunnelProviderManagerType) {
        guard self.tunnelProvider != tunnelProvider else {
            return
        }

        // Save the new active tunnel provider
        self.tunnelProvider = tunnelProvider

        // Set up tunnel IPC
        self.ipcSession = TunnelIPC.Session(from: tunnelProvider)

        // Register for tunnel connection status changes
        unregisterConnectionObserver()
        connectionStatusObserver = NotificationCenter.default
            .addObserver(forName: .NEVPNStatusDidChange, object: tunnelProvider.connection, queue: nil) {
                [weak self] (notification) in
                guard let self = self else { return }

                self.stateQueue.async {
                    self.updateTunnelState()
                }
        }

        // Update the existing state
        updateTunnelState()
    }

    private func unregisterConnectionObserver() {
        if let connectionStatusObserver = connectionStatusObserver {
            NotificationCenter.default.removeObserver(connectionStatusObserver)
            self.connectionStatusObserver = nil
        }
    }

    private func pushWireguardKeyAndUpdateSettings(accountToken: String, publicKey: PublicKey) -> Result<TunnelSettings, Error>.Promise {
        return REST.Client.shared.pushWireguardKey(token: accountToken, publicKey: publicKey)
            .mapError { error in
                return .pushWireguardKey(error)
            }
            .receive(on: stateQueue)
            .flatMap { associatedAddresses in
                return Self.updateTunnelSettings(accountToken: accountToken) { (tunnelSettings) in
                    tunnelSettings.interface.addresses = [
                        associatedAddresses.ipv4Address,
                        associatedAddresses.ipv6Address
                    ]
                }
            }
    }

    private func removeWireguardKeyFromServer(accountToken: String, publicKey: PublicKey) -> Result<Bool, Error>.Promise {
        return REST.Client.shared.deleteWireguardKey(token: accountToken, publicKey: publicKey)
            .map { _ in
                return true
            }
            .flatMapError { restError -> Result<Bool, Error> in
                if case .server(.pubKeyNotFound) = restError {
                    return .success(false)
                } else {
                    return .failure(.removeWireguardKey(restError))
                }
            }
    }

    private func replaceWireguardKeyAndUpdateSettings(
        accountToken: String,
        oldPublicKey: PublicKeyWithMetadata,
        newPrivateKey: PrivateKeyWithMetadata
    ) -> Result<TunnelSettings, Error>.Promise
    {
        return REST.Client.shared.replaceWireguardKey(token: accountToken, oldPublicKey: oldPublicKey.publicKey, newPublicKey: newPrivateKey.publicKey)
            .mapError { error in
                return .replaceWireguardKey(error)
            }
            .receive(on: self.stateQueue)
            .flatMap { associatedAddresses in
                return Self.updateTunnelSettings(accountToken: accountToken) { (tunnelSettings) in
                    tunnelSettings.interface.privateKey = newPrivateKey
                    tunnelSettings.interface.addresses = [
                        associatedAddresses.ipv4Address,
                        associatedAddresses.ipv6Address
                    ]
                }
            }
    }

    /// Update `TunnelState` from `NEVPNStatus`.
    /// Collects the `TunnelConnectionInfo` from the tunnel via IPC if needed before assigning the `tunnelState`
    private func updateTunnelState() {
        dispatchPrecondition(condition: .onQueue(stateQueue))

        guard let connectionStatus = self.tunnelProvider?.connection.status else { return }

        logger.debug("VPN status changed to \(connectionStatus)")
        tunnelConnectionInfoToken = nil

        switch connectionStatus {
        case .connecting:
            switch tunnelState {
            case .connecting(.some(_)):
                logger.debug("Ignore repeating connecting state.")
            default:
                tunnelState = .connecting(nil)
            }

        case .reasserting:
            ipcSession?.getTunnelConnectionInfo()
                .receive(on: stateQueue)
                .storeCancellationToken(in: &tunnelConnectionInfoToken)
                .onSuccess { connectionInfo in
                    if let connectionInfo = connectionInfo {
                        self.tunnelState = .reconnecting(connectionInfo)
                    }
                }
                .observe { _ in }

        case .connected:
            ipcSession?.getTunnelConnectionInfo()
                .receive(on: stateQueue)
                .storeCancellationToken(in: &tunnelConnectionInfoToken)
                .onSuccess { connectionInfo in
                    if let connectionInfo = connectionInfo {
                        self.tunnelState = .connected(connectionInfo)
                    }
                }
                .observe { _ in }

        case .disconnected:
            switch tunnelState {
            case .pendingReconnect:
                logger.debug("Ignore disconnected state when pending reconnect.")

            case .disconnecting(.reconnect):
                logger.debug("Restart the tunnel on disconnect.")
                tunnelState = .pendingReconnect
                startTunnel()

            default:
                tunnelState = .disconnected
            }

        case .disconnecting:
            switch tunnelState {
            case .disconnecting:
                break
            default:
                tunnelState = .disconnecting(.nothing)
            }

        case .invalid:
            tunnelState = .disconnected

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
        }
    }

    private func makeTunnelProvider(accountToken: String) -> Result<TunnelProviderManagerType, Error>.Promise {
        return TunnelProviderManagerType.loadAllFromPreferences()
            .mapError { error -> Error in
                return .loadAllVPNConfigurations(error)
            }
            .flatMap { tunnels in
                return Self.setupTunnelProvider(accountToken: accountToken, tunnels: tunnels)
            }
            .mapThen { tunnelProvider in
                return tunnelProvider.saveToPreferences()
                    .mapError { error in
                        return .saveVPNConfiguration(error)
                    }
                    .mapThen { _ in
                        // Refresh connection status after saving the tunnel preferences.
                        // Basically it's only necessary to do for new instances of
                        // `NETunnelProviderManager`, but we do that for the existing ones too
                        // for simplicity as it has no side effects.
                        return tunnelProvider.loadFromPreferences()
                            .mapError { error in
                                return .reloadVPNConfiguration(error)
                            }
                    }
                    .setOutput(tunnelProvider)
            }
    }

    private func sendFailureToObservers(_ failure: Error) {
        DispatchQueue.main.async {
            self.observerList.forEach { observer in
                observer.tunnelManager(self, didFailWithError: failure)
            }
        }
    }

    private func notifyTunnelOnSettingsChange() -> Promise<Void> {
        return Promise.deferred { () -> (TunnelIPC.Session, TunnelProviderManagerType)? in
            if let ipcSession = self.ipcSession, let tunnelProvider = self.tunnelProvider {
                return (ipcSession, tunnelProvider)
            } else {
                return nil
            }
        }
        .mapThen(defaultValue: ()) { (ipc, tunnelProvider) in
            return Promise { resolver in
                let connection = tunnelProvider.connection
                var statusObserver: NSObjectProtocol?
                var ipcToken: PromiseCancellationToken?

                let releaseObserver = {
                    if let statusObserver = statusObserver {
                        NotificationCenter.default.removeObserver(statusObserver)
                    }
                }

                let handleStatus = {
                    switch connection.status {
                    case .connected:
                        releaseObserver()

                        ipc.reloadTunnelSettings()
                            .storeCancellationToken(in: &ipcToken)
                            .observe { completion in
                                switch completion {
                                case .finished(let result):
                                    if case .failure(let error) = result {
                                        self.logger.error(chainedError: error, message: "Failed to send IPC request to reload tunnel settings")
                                    }
                                    resolver.resolve(value: ())
                                case .cancelled:
                                    resolver.resolve(completion: .cancelled)
                                }
                            }

                    case .connecting, .reasserting:
                        // wait for transition to complete
                        break

                    case .invalid, .disconnecting, .disconnected:
                        releaseObserver()
                        resolver.resolve(value: ())

                    @unknown default:
                        break
                    }
                }

                // Add connection status observer
                statusObserver = NotificationCenter.default.addObserver(
                    forName: .NEVPNStatusDidChange,
                    object: connection,
                    queue: .main) { note in
                        handleStatus()
                    }

                // Set cancellation handler
                resolver.setCancelHandler {
                    DispatchQueue.main.async {
                        releaseObserver()
                        ipcToken = nil
                    }
                }

                // Run initial check
                DispatchQueue.main.async {
                    handleStatus()
                }
            }
        }
        .schedule(on: stateQueue)
        .run(on: operationQueue, categories: [OperationCategory.notifyTunnelSettingsChange])
        .requestBackgroundTime(taskName: "TunnelManager.notifyTunnelOnSettingsChange")
    }

    @objc private func applicationDidBecomeActive() {
        stateQueue.async {
            // Refresh tunnel state when application becomes active.
            self.updateTunnelState()
        }
    }

    // MARK: - Private class methods

    private class func loadTunnelSettings(accountToken: String) -> Result<TunnelSettingsManager.KeychainEntry, Error> {
        return TunnelSettingsManager.load(searchTerm: .accountToken(accountToken))
            .mapError { Error.readTunnelSettings($0) }
    }

    private class func updateTunnelSettings(accountToken: String, block: (inout TunnelSettings) -> Void) -> Result<TunnelSettings, Error> {
        return TunnelSettingsManager.update(searchTerm: .accountToken(accountToken), using: block)
            .mapError { Error.updateTunnelSettings($0) }
    }

    /// Retrieve the existing `TunnelSettings` or create the new ones
    private class func makeTunnelSettings(accountToken: String) -> Result<TunnelSettings, Error> {
        return Self.loadTunnelSettings(accountToken: accountToken)
            .map { $0.tunnelSettings }
            .flatMapError { error in
                if case .readTunnelSettings(.lookupEntry(.itemNotFound)) = error {
                    let defaultConfiguration = TunnelSettings()

                    return TunnelSettingsManager
                        .add(configuration: defaultConfiguration, account: accountToken)
                        .mapError { .addTunnelSettings($0) }
                        .map { defaultConfiguration }
                } else {
                    return .failure(error)
                }
            }
    }

    private class func setupTunnelProvider(accountToken: String ,tunnels: [TunnelProviderManagerType]?) -> Result<TunnelProviderManagerType, Error> {
        // Request persistent keychain reference to tunnel settings
        return TunnelSettingsManager.getPersistentKeychainReference(account: accountToken)
            .map { (passwordReference) -> TunnelProviderManagerType in
                // Get the first available tunnel or make a new one
                let tunnelProvider = tunnels?.first ?? TunnelProviderManagerType()

                let protocolConfig = NETunnelProviderProtocol()
                protocolConfig.providerBundleIdentifier = ApplicationConfiguration.packetTunnelExtensionIdentifier
                protocolConfig.serverAddress = ""
                protocolConfig.username = accountToken
                protocolConfig.passwordReference = passwordReference

                tunnelProvider.isEnabled = true
                tunnelProvider.localizedDescription = "WireGuard"
                tunnelProvider.protocolConfiguration = protocolConfig

                // Enable on-demand VPN, always connect the tunnel when on Wi-Fi or cellular
                let alwaysOnRule = NEOnDemandRuleConnect()
                alwaysOnRule.interfaceTypeMatch = .any
                tunnelProvider.onDemandRules = [alwaysOnRule]
                tunnelProvider.isOnDemandEnabled = true

                return tunnelProvider
            }.mapError { (error) -> Error in
                return .obtainPersistentKeychainReference(error)
            }
    }

    private func migrateTunnelSettings(accountToken: String) -> Result<Bool, Error> {
        let result = TunnelSettingsManager
            .migrateKeychainEntry(searchTerm: .accountToken(accountToken))
            .mapError { (error) -> Error in
                return .migrateTunnelSettings(error)
            }

        switch result {
        case .success(let migrated):
            if migrated {
                self.logger.info("Migrated Keychain tunnel configuration.")
            } else {
                self.logger.info("Tunnel settings are up to date. No migration needed.")
            }

        case .failure(let error):
            self.logger.error(chainedError: error)
        }

        return result
    }

}

extension TunnelManager {
    /// Key rotation result.
    enum KeyRotationResult: CustomStringConvertible {
        /// Request to rotate the key was throttled.
        case throttled(_ lastKeyCreationDate: Date)

        /// New key was generated.
        case finished

        var description: String {
            switch self {
            case .throttled:
                return "throttled"
            case .finished:
                return "finished"
            }
        }
    }
}

extension TunnelManager {
    fileprivate func handlePrivateKeyRotationCompletion(completion: PromiseCompletion<Result<KeyRotationResult, TunnelManager.Error>>) -> Date? {
        switch completion {
        case .finished(.success(let result)):
            switch result {
            case .finished:
                self.logger.debug("Finished private key rotation")
            case .throttled:
                self.logger.debug("Private key was already rotated earlier")
            }

            return self.nextScheduleDate(result)

        case .finished(.failure(let error)):
            self.logger.error(chainedError: error, message: "Failed to rotate private key in background task")

            return self.nextRetryScheduleDate(error)

        case .cancelled:
            self.logger.debug("Private key rotation was cancelled")

            return Date(timeIntervalSinceNow: Self.privateKeyRotationFailureRetryInterval)
        }
    }

    fileprivate func nextScheduleDate(_ result: KeyRotationResult) -> Date {
        switch result {
        case .finished:
            return Date(timeIntervalSinceNow: Self.privateKeyRotationInterval)

        case .throttled(let lastKeyCreationDate):
            return Date(timeInterval: Self.privateKeyRotationInterval, since: lastKeyCreationDate)
        }
    }

    fileprivate func nextRetryScheduleDate(_ error: TunnelManager.Error) -> Date? {
        switch error {
        case .missingAccount:
            // Do not retry if logged out.
            return nil

        case .replaceWireguardKey(.server(.invalidAccount)):
            // Do not retry if account was removed.
            return nil

        default:
            return Date(timeIntervalSinceNow: Self.privateKeyRotationFailureRetryInterval)
        }
    }
}
