//
//  RenegeratePrivateKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol RenegeratePrivateKeyOperationDelegate: AnyObject {
    func operationDidRequestTunnelInfo(_ operation: Operation) -> TunnelInfo?
    func operation(_ operation: Operation, didFinishRegeneratingPrivateKeyWithCompletion completion: OperationCompletion<TunnelSettings, TunnelManager.Error>)
}

class RenegeratePrivateKeyOperation: BaseTunnelOperation<TunnelSettings, TunnelManager.Error> {
    private let restClient: REST.Client
    private weak var delegate: RenegeratePrivateKeyOperationDelegate?
    private var restCancellable: Cancellable?

    init(queue: DispatchQueue, restClient: REST.Client, delegate: RenegeratePrivateKeyOperationDelegate) {
        self.restClient = restClient
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

                    self.completeOperation(completion: OperationCompletion(result: saveResult))
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

    override func completeOperation(completion: OperationCompletion<TunnelSettings, TunnelManager.Error>) {
        delegate?.operation(self, didFinishRegeneratingPrivateKeyWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }
}
