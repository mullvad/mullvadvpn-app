//
//  AutomaticKeyRotationManager.swift
//  MullvadVPN
//
//  Created by pronebird on 05/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import WireGuardKit

/// A private key rotation retry interval on failure (in seconds)
private let kRetryIntervalOnFailure = 300

/// A private key rotation interval (in days)
private let kRotationInterval = 4

/// A struct describing the key rotation result
struct KeyRotationResult {
    var isNew: Bool
    var publicKeyWithMetadata: PublicKeyWithMetadata
}

class AutomaticKeyRotationManager {

    enum Error: ChainedError {
        /// REST error
        case rest(RestError)

        /// A failure to read the tunnel settings
        case readTunnelSettings(TunnelSettingsManager.Error)

        /// A failure to update tunnel settings
        case updateTunnelSettings(TunnelSettingsManager.Error)

        var errorDescription: String? {
            switch self {
            case .rest:
                return "REST error"
            case .readTunnelSettings:
                return "Read tunnel settings error"
            case .updateTunnelSettings:
                return "Update tunnel settings error"
            }
        }
    }

    private let logger = Logger(label: "AutomaticKeyRotationManager")

    private let rest = MullvadRest()
    private let persistentKeychainReference: Data

    /// A dispatch queue used for synchronization
    private let dispatchQueue = DispatchQueue(label: "net.mullvad.vpn.key-manager", qos: .utility)

    /// A timer source used to schedule a delayed key rotation
    private var timerSource: DispatchSourceTimer?

    /// Internal lock used for access synchronization to public members of this class
    private let stateLock = NSLock()

    /// Internal variable indicating that the key rotation has already started
    private var isAutomaticRotationEnabled = false

    /// A REST request for replacing the key on server
    private var dataTask: URLSessionTask?

    /// A variable backing the `eventHandler` public property
    private var _eventHandler: ((KeyRotationResult) -> Void)?

    /// A dispatch queue used for broadcasting events
    private let eventQueue: DispatchQueue?

    /// An event handler that's invoked when key rotation occurred
    var eventHandler: ((KeyRotationResult) -> Void)? {
        get {
            stateLock.withCriticalBlock {
                self._eventHandler
            }
        }
        set {
            stateLock.withCriticalBlock {
                self._eventHandler = newValue
            }
        }
    }

    init(persistentKeychainReference: Data, eventQueue: DispatchQueue?) {
        self.persistentKeychainReference = persistentKeychainReference
        self.eventQueue = eventQueue
    }

    func startAutomaticRotation(queue: DispatchQueue?, completionHandler: @escaping () -> Void) {
        dispatchQueue.async {
            if !self.isAutomaticRotationEnabled {
                self.logger.info("Start automatic key rotation")

                self.isAutomaticRotationEnabled = true
                self.performKeyRotation()
            }

            queue.performOnWrappedOrCurrentQueue(block: completionHandler)
        }
    }

    func stopAutomaticRotation(queue: DispatchQueue?, completionHandler: @escaping () -> Void) {
        dispatchQueue.async {
            if self.isAutomaticRotationEnabled {
                self.logger.info("Stop automatic key rotation")

                self.isAutomaticRotationEnabled = false

                self.dataTask?.cancel()
                self.dataTask = nil

                self.timerSource?.cancel()
            }

            queue.performOnWrappedOrCurrentQueue(block: completionHandler)
        }
    }

    private func performKeyRotation() {
        let result = TunnelSettingsManager.load(searchTerm: .persistentReference(persistentKeychainReference))

        switch result {
        case .success(let keychainEntry):
            let currentPrivateKey = keychainEntry.tunnelSettings.interface.privateKey

            if Self.shouldRotateKey(creationDate: currentPrivateKey.creationDate) {
                let result = makeReplaceKeyTask(accountToken: keychainEntry.accountToken, oldPublicKey: currentPrivateKey.privateKey.publicKey) { (result) in
                    let result = result.map { (tunnelSettings) -> KeyRotationResult in
                        let newPrivateKey = tunnelSettings.interface.privateKey

                        return KeyRotationResult(
                            isNew: true,
                            publicKeyWithMetadata: newPrivateKey.publicKeyWithMetadata
                        )
                    }

                    self.didCompleteKeyRotation(result: result)
                }

                switch result {
                case .success(let newTask):
                    self.dataTask = newTask
                    newTask.resume()

                case .failure(let error):
                    self.dataTask = nil
                    self.didCompleteKeyRotation(result: .failure(.rest(error)))
                }
            } else {
                let event = KeyRotationResult(
                    isNew: false,
                    publicKeyWithMetadata: currentPrivateKey.publicKeyWithMetadata
                )

                self.didCompleteKeyRotation(result: .success(event))
            }

        case .failure(let error):
            self.didCompleteKeyRotation(result: .failure(.readTunnelSettings(error)))
        }
    }

    private func makeReplaceKeyTask(
        accountToken: String,
        oldPublicKey: PublicKey,
        completionHandler: @escaping (Result<TunnelSettings, Error>) -> Void) -> Result<URLSessionDataTask, RestError>
    {
        let newPrivateKeyWithMetadata = PrivateKeyWithMetadata()
        let payload = TokenPayload(
            token: accountToken,
            payload: ReplaceWireguardKeyRequest(
                old: oldPublicKey.rawValue,
                new: newPrivateKeyWithMetadata.privateKey.publicKey.rawValue
            )
        )

        return rest.replaceWireguardKey().dataTask(payload: payload) { (result) in
            self.dispatchQueue.async {
                let updateResult = result.mapError { (error) -> Error in
                    return .rest(error)
                }.flatMap { (response) -> Result<TunnelSettings, Error> in
                    let addresses = WireguardAssociatedAddresses(
                        ipv4Address: response.ipv4Address,
                        ipv6Address: response.ipv6Address
                    )

                    return self.updateTunnelSettings(privateKeyWithMetadata: newPrivateKeyWithMetadata, addresses: addresses)
                }
                completionHandler(updateResult)
            }
        }
    }

    private func updateTunnelSettings(privateKeyWithMetadata: PrivateKeyWithMetadata, addresses: WireguardAssociatedAddresses) -> Result<TunnelSettings, Error> {
        let updateResult = TunnelSettingsManager.update(searchTerm: .persistentReference(self.persistentKeychainReference))
            { (tunnelSettings) in
                tunnelSettings.interface.privateKey = privateKeyWithMetadata
                tunnelSettings.interface.addresses = [
                    addresses.ipv4Address,
                    addresses.ipv6Address
                ]
        }

        return updateResult.mapError { .updateTunnelSettings($0) }
    }

    private func didCompleteKeyRotation(result: Result<KeyRotationResult, Error>) {
        var nextRotationTime: DispatchWallTime?

        switch result {
        case .success(let event):
            if event.isNew {
                logger.info("Finished private key rotation")

                eventQueue.performOnWrappedOrCurrentQueue {
                    self.eventHandler?(event)
                }
            }

            if let rotationDate = Self.nextRotation(creationDate: event.publicKeyWithMetadata.creationDate) {
                let interval = rotationDate.timeIntervalSinceNow

                logger.info("Next private key rotation on \(rotationDate)")

                nextRotationTime = .now() + .seconds(Int(interval))
            } else {
                logger.error("Failed to compute the next private rotation date. Retry in \(kRetryIntervalOnFailure) seconds.")

                nextRotationTime = .now() + .seconds(kRetryIntervalOnFailure)
            }

        case .failure(.rest(.network(URLError.cancelled))):
            logger.info("Key rotation was cancelled")

        case .failure(let error):
            logger.error("Failed to rotate the private key. Retry in \(kRetryIntervalOnFailure) seconds. Error: \(error.displayChain())")

            nextRotationTime = .now() + .seconds(kRetryIntervalOnFailure)
        }

        if let nextRotationTime = nextRotationTime, isAutomaticRotationEnabled {
            scheduleRetry(wallDeadline: nextRotationTime)
        }
    }

    private func scheduleRetry(wallDeadline: DispatchWallTime) {
        let timerSource = DispatchSource.makeTimerSource(queue: dispatchQueue)
        timerSource.setEventHandler { [weak self] in
            guard let self = self else { return }

            if self.isAutomaticRotationEnabled {
                self.performKeyRotation()
            }
        }

        timerSource.schedule(wallDeadline: wallDeadline)
        timerSource.activate()

        self.timerSource = timerSource
    }

    private class func nextRotation(creationDate: Date) -> Date? {
        return Calendar.current.date(byAdding: .day, value: kRotationInterval, to: creationDate)
    }

    private class func shouldRotateKey(creationDate: Date) -> Bool {
        return nextRotation(creationDate: creationDate)
            .map { $0 <= Date() } ?? false
    }

}
