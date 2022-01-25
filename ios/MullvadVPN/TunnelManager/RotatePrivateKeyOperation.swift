//
//  RotatePrivateKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol RotatePrivateKeyOperationDelegate: AnyObject {
    func operationDidRequestTunnelInfo(_ operation: Operation) -> TunnelInfo?
    func operation(_ operation: Operation, didSaveTunnelSettings newTunnelSettings: TunnelSettings)
    func operation(_ operation: Operation, didFinishKeyRotationWithCompletion completion: OperationCompletion<TunnelManager.KeyRotationResult, TunnelManager.Error>)
}

class RotatePrivateKeyOperation: BaseTunnelOperation<TunnelManager.KeyRotationResult, TunnelManager.Error> {
    private let restClient: REST.Client
    private let rotationInterval: TimeInterval
    private weak var delegate: RotatePrivateKeyOperationDelegate?
    private var restCancellable: Cancellable?

    init(queue: DispatchQueue,
         restClient: REST.Client,
         rotationInterval: TimeInterval,
         delegate: RotatePrivateKeyOperationDelegate)
    {
        self.restClient = restClient
        self.rotationInterval = rotationInterval
        self.delegate = delegate

        super.init(queue: queue)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.completeOperation(completion: .cancelled)
                return
            }

            guard let tunnelInfo = self.delegate?.operationDidRequestTunnelInfo(self) else {
                self.completeOperation(completion: .failure(.missingAccount))
                return
            }

            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let timeInterval = Date().timeIntervalSince(creationDate)

            guard timeInterval >= self.rotationInterval else {
                self.completeOperation(completion: .success(.throttled(creationDate)))
                return
            }

            let newPrivateKey = PrivateKeyWithMetadata()
            let oldPublicKey = tunnelInfo.tunnelSettings.interface.publicKey

            let restRequest = self.restClient.replaceWireguardKey(
                token: tunnelInfo.token,
                oldPublicKey: oldPublicKey,
                newPublicKey: newPrivateKey.publicKey
            )

            self.restCancellable = restRequest.execute { restResult in
                self.queue.async {
                    let saveResult = Self.handleResponse(token: tunnelInfo.token, newPrivateKey: newPrivateKey, result: restResult)

                    switch saveResult {
                    case .success(let tunnelSettings):
                        self.delegate?.operation(self, didSaveTunnelSettings: tunnelSettings)
                        self.completeOperation(completion: .success(.finished))

                    case .failure(let error):
                        self.completeOperation(completion: .failure(error))
                    }
                }
            }
        }
    }

    override func cancel() {
        super.cancel()

        queue.async {
            self.restCancellable?.cancel()
        }
    }

    override func completeOperation(completion: OperationCompletion<TunnelManager.KeyRotationResult, TunnelManager.Error>) {
        delegate?.operation(self, didFinishKeyRotationWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }

    private class func handleResponse(token: String, newPrivateKey: PrivateKeyWithMetadata, result: Result<REST.WireguardAddressesResponse, REST.Error>) -> Result<TunnelSettings, TunnelManager.Error> {
        return result.flatMapError { restError in
            return .failure(.replaceWireguardKey(restError))
        }
        .flatMap { associatedAddresses in
            return TunnelSettingsManager.update(searchTerm: .accountToken(token)) { newTunnelSettings in
                newTunnelSettings.interface.privateKey = newPrivateKey
                newTunnelSettings.interface.addresses = [
                    associatedAddresses.ipv4Address,
                    associatedAddresses.ipv6Address
                ]
            }.mapError { error -> TunnelManager.Error in
                return .updateTunnelSettings(error)
            }
        }
    }
}
