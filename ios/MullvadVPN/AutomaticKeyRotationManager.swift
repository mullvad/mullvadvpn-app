//
//  AutomaticKeyRotationManager.swift
//  MullvadVPN
//
//  Created by pronebird on 05/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import os

/// A private key rotation retry interval on failure (in seconds)
private let kRetryIntervalOnFailure = 300

/// A private key rotation interval (in days)
private let kRotationInterval = 4

class AutomaticKeyRotationManager {

    enum Error: Swift.Error {
        /// An RPC failure
        case rpc(MullvadRpc.Error)

        /// A failure to read the tunnel configuration
        case readTunnelConfiguration(TunnelConfigurationManager.Error)

        /// A failure to update tunnel configuration
        case updateTunnelConfiguration(TunnelConfigurationManager.Error)

        var localizedDescription: String {
            switch self {
            case .rpc(let error):
                return "Rpc error: \(error.localizedDescription)"
            case .readTunnelConfiguration(let error):
                return "Read configuration error: \(error.localizedDescription)"
            case .updateTunnelConfiguration(let error):
                return "Update configuration error: \(error.localizedDescription)"
            }
        }
    }

    struct KeyRotationEvent {
        var isNew: Bool
        var creationDate: Date
        var publicKey: WireguardPublicKey
    }

    private let rpc = MullvadRpc()
    private let persistentKeychainReference: Data
    private var rotateKeySubscriber: AnyCancellable?

    /// A dispatch queue used for synchronization
    private let dispatchQueue = DispatchQueue(label: "net.mullvad.vpn.key-manager", qos: .background)

    /// A timer source used to schedule a delayed key rotation
    private var timerSource: DispatchSourceTimer?

    /// Internal lock used for access synchronization to public members of this class
    private let lock = NSLock()

    /// Internal variable indicating that the key rotation has already started
    private var isAutomaticRotationEnabled = false

    /// A variable backing the `eventHandler` public property
    private var _eventHandler: ((KeyRotationEvent) -> Void)?

    /// An event handler that's invoked when key rotation occurred
    var eventHandler: ((KeyRotationEvent) -> Void)? {
        get {
            lock.withCriticalBlock {
                self._eventHandler
            }
        }
        set {
            lock.withCriticalBlock {
                self._eventHandler = newValue
            }
        }
    }

    init(persistentKeychainReference: Data) {
        self.persistentKeychainReference = persistentKeychainReference
    }

    func startAutomaticRotation() {
        dispatchQueue.async {
            guard !self.isAutomaticRotationEnabled else { return }

            os_log(.default, log: tunnelProviderLog, "Start automatic key rotation")

            self.isAutomaticRotationEnabled = true
            self.performKeyRotation()
        }
    }

    func stopAutomaticRotation() {
        dispatchQueue.async {
            guard self.isAutomaticRotationEnabled else { return }

            os_log(.default, log: tunnelProviderLog, "Stop automatic key rotation")

            self.isAutomaticRotationEnabled = false
            self.rotateKeySubscriber?.cancel()
            self.timerSource?.cancel()
        }
    }

    private func performKeyRotation() {
        rotateKeySubscriber = tryRotatingPrivateKey()
            .receive(on: dispatchQueue)
            .sink(receiveCompletion: { [weak self] (completion) in
                guard let self = self else { return }

                switch completion {
                case .finished:
                    break

                case .failure(let error):
                    os_log(.error, log: tunnelProviderLog,
                           "Failed to rotate the private key: %{public}s. Retry in %d seconds.",
                           error.localizedDescription,
                           kRetryIntervalOnFailure)

                    self.scheduleRetry(wallDeadline: .now() + .seconds(kRetryIntervalOnFailure))
                }
            }) { [weak self] (keyRotationEvent) in
                guard let self = self else { return }

                if keyRotationEvent.isNew {
                    os_log(.default, log: tunnelProviderLog, "Finished private key rotation")

                    self.eventHandler?(keyRotationEvent)
                }

                if let rotationDate = Self.nextRotation(creationDate: keyRotationEvent.creationDate) {
                    let interval = rotationDate.timeIntervalSinceNow

                    os_log(.default, log: tunnelProviderLog,
                           "Next private key rotation on %{public}s", "\(rotationDate)")

                    self.scheduleRetry(wallDeadline: .now() + .seconds(Int(interval)))
                } else {
                    os_log(.error, log: tunnelProviderLog,
                           "Failed to compute the next private rotation date. Retry in %d seconds.")

                    self.scheduleRetry(wallDeadline: .now() + .seconds(kRetryIntervalOnFailure))
                }
        }
    }

    private func scheduleRetry(wallDeadline: DispatchWallTime) {
        let timerSource = DispatchSource.makeTimerSource(queue: dispatchQueue)
        timerSource.setEventHandler { [weak self] in
            self?.performKeyRotation()
        }

        timerSource.schedule(wallDeadline: wallDeadline)
        timerSource.activate()

        self.timerSource = timerSource
    }

    private func tryRotatingPrivateKey() -> AnyPublisher<KeyRotationEvent, Error> {
        return TunnelConfigurationManager
            .load(searchTerm: .persistentReference(persistentKeychainReference))
            .mapError { .readTunnelConfiguration($0) }
            .publisher
            .flatMap { (keychainEntry) -> AnyPublisher<KeyRotationEvent, Error> in
                let currentPrivateKey = keychainEntry.tunnelConfiguration.interface.privateKey

                if Self.shouldRotateKey(creationDate: currentPrivateKey.creationDate) {
                    return self.replaceWireguardKey(
                        accountToken: keychainEntry.accountToken,
                        oldPublicKey: currentPrivateKey.publicKey
                    ).map({ (newTunnelConfiguration) -> KeyRotationEvent in
                        let newPrivateKey = newTunnelConfiguration.interface.privateKey

                        return KeyRotationEvent(
                            isNew: true,
                            creationDate: newPrivateKey.creationDate,
                            publicKey: newPrivateKey.publicKey
                        )
                    }).eraseToAnyPublisher()
                } else {
                    let result = KeyRotationEvent(
                        isNew: false,
                        creationDate: currentPrivateKey.creationDate,
                        publicKey: currentPrivateKey.publicKey
                    )

                    return Result.Publisher(result).eraseToAnyPublisher()
                }
        }.eraseToAnyPublisher()
    }

    private func replaceWireguardKey(accountToken: String, oldPublicKey: WireguardPublicKey)
        -> AnyPublisher<TunnelConfiguration, Error>
    {
        let newPrivateKey = WireguardPrivateKey()

        return rpc.replaceWireguardKey(
            accountToken: accountToken,
            oldPublicKey: oldPublicKey.rawRepresentation,
            newPublicKey: newPrivateKey.publicKey.rawRepresentation)
            .mapError {  .rpc($0) }
            .flatMap { (addresses) in
                TunnelConfigurationManager
                    .update(searchTerm: .persistentReference(self.persistentKeychainReference))
                    { (tunnelConfiguration) in
                        tunnelConfiguration.interface.privateKey = newPrivateKey
                        tunnelConfiguration.interface.addresses = [
                            addresses.ipv4Address,
                            addresses.ipv6Address
                        ]
                }
                .mapError { .updateTunnelConfiguration($0) }
                .publisher
        }.eraseToAnyPublisher()
    }

    class func nextRotation(creationDate: Date) -> Date? {
        return Calendar.current.date(byAdding: .day, value: kRotationInterval, to: creationDate)
    }

    class func shouldRotateKey(creationDate: Date) -> Bool {
        return nextRotation(creationDate: creationDate)
            .map { $0 <= Date() } ?? false
    }
}
