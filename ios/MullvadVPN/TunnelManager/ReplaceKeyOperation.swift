//
//  ReplaceKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class ReplaceKeyOperation: ResultOperation<TunnelManager.KeyRotationResult, TunnelManager.Error> {
    private let state: TunnelManager.State

    private let apiProxy: REST.APIProxy
    private var restRequest: Cancellable?

    private let rotationInterval: TimeInterval?

    private let logger = Logger(label: "TunnelManager.ReplaceKeyOperation")

    class func operationForKeyRotation(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        apiProxy: REST.APIProxy,
        rotationInterval: TimeInterval,
        completionHandler: @escaping CompletionHandler
    ) -> ReplaceKeyOperation {
        return ReplaceKeyOperation(
            dispatchQueue: dispatchQueue,
            state: state,
            apiProxy: apiProxy,
            rotationInterval: rotationInterval,
            completionHandler: completionHandler
        )
    }

    class func operationForKeyRegeneration(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        apiProxy: REST.APIProxy,
        completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void
    ) -> ReplaceKeyOperation {
        return ReplaceKeyOperation(
            dispatchQueue: dispatchQueue,
            state: state,
            apiProxy: apiProxy,
            rotationInterval: nil
        ) { completion in
            let mappedCompletion = completion.map { keyRotationResult -> () in
                switch keyRotationResult {
                case .finished:
                    return ()
                case .throttled:
                    fatalError("ReplaceKeyOperation.operationForKeyRegeneration() must never recieve throttled!")
                }
            }

            completionHandler(mappedCompletion)
        }
    }

    private init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        apiProxy: REST.APIProxy,
        rotationInterval: TimeInterval?,
        completionHandler: @escaping CompletionHandler
    ) {
        self.state = state

        self.apiProxy = apiProxy
        self.rotationInterval = rotationInterval

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        self.execute { completion in
            self.finish(completion: completion)
        }
    }

    override func operationDidCancel() {
        restRequest?.cancel()
        restRequest = nil
    }

    private func execute(completionHandler: @escaping CompletionHandler) {
        guard let tunnelInfo = state.tunnelInfo else {
            completionHandler(.failure(.unsetAccount))
            return
        }

        if let rotationInterval = rotationInterval {
            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let nextRotationDate = creationDate.addingTimeInterval(rotationInterval)

            if nextRotationDate > Date() {
                logger.debug("Throttle private key rotation.")

                completionHandler(.success(.throttled(creationDate)))
                return
            } else {
                logger.debug("Private key is old enough, rotate right away.")
            }
        } else {
            logger.debug("Rotate private key right away.")
        }

        let newPrivateKey: PrivateKeyWithMetadata
        let oldPublicKey = tunnelInfo.tunnelSettings.interface.publicKey

        if let nextPrivateKey = tunnelInfo.tunnelSettings.interface.nextPrivateKey {
            newPrivateKey = nextPrivateKey

            logger.debug("Next private key is already created.")
        } else {
            newPrivateKey = PrivateKeyWithMetadata()

            logger.debug("Create next private key.")

            let saveResult = TunnelSettingsManager.update(searchTerm: .accountToken(tunnelInfo.token)) { newTunnelSettings in
                newTunnelSettings.interface.nextPrivateKey = newPrivateKey
            }

            switch saveResult {
            case .success(let newTunnelSettings):
                logger.debug("Saved next private key.")

                state.tunnelInfo?.tunnelSettings = newTunnelSettings

            case .failure(let error):
                logger.error(chainedError: error, message: "Failed to save next private key.")

                completionHandler(.failure(.updateTunnelSettings(error)))
                return
            }
        }

        logger.debug("Replacing old key with new key on server...")

        restRequest = self.apiProxy.replaceWireguardKey(
            accountNumber: tunnelInfo.token,
            oldPublicKey: oldPublicKey,
            newPublicKey: newPrivateKey.publicKey,
            retryStrategy: .default
        ) { completion in
            self.dispatchQueue.async {
                self.didReceiveResponse(
                    completion: completion,
                    accountToken: tunnelInfo.token,
                    newPrivateKey: newPrivateKey,
                    completionHandler: completionHandler
                )
            }
        }
    }

    private func didReceiveResponse(completion: OperationCompletion<REST.WireguardAddressesResponse, REST.Error>, accountToken: String, newPrivateKey: PrivateKeyWithMetadata, completionHandler: @escaping CompletionHandler) {
        switch completion {
        case .success(let associatedAddresses):
            logger.debug("Replaced old key with new key on server.")

            let saveResult = TunnelSettingsManager.update(searchTerm: .accountToken(accountToken)) { newTunnelSettings in
                newTunnelSettings.interface.privateKey = newPrivateKey
                newTunnelSettings.interface.nextPrivateKey = nil

                newTunnelSettings.interface.addresses = [
                    associatedAddresses.ipv4Address,
                    associatedAddresses.ipv6Address
                ]
            }

            switch saveResult {
            case .success(let newTunnelSettings):
                logger.debug("Saved associated addresses.")

                state.tunnelInfo?.tunnelSettings = newTunnelSettings

                completionHandler(.success(.finished))

            case .failure(let error):
                logger.error(chainedError: error, message: "Failed to save associated addresses.")

                completionHandler(.failure(.updateTunnelSettings(error)))
            }

        case .failure(let restError):
            logger.error(chainedError: restError, message: "Failed to replace old key with new key on server.")

            completionHandler(.failure(.replaceWireguardKey(restError)))

        case .cancelled:
            logger.debug("Cancelled replace key request.")

            completionHandler(.cancelled)
        }
    }
}
