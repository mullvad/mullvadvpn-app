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
    func operation(_ operation: Operation, didFinishRotatingPrivateKeyWithNewTunnelSettings newTunnelSettings: TunnelSettings)
    func operation(_ operation: Operation, didFailToRotatePrivateKeyWithError error: TunnelManager.Error)
}

class RotatePrivateKeyOperation: AsyncOperation {
    typealias CompletionHandler = (TunnelManager.KeyRotationResult?, TunnelManager.Error?) -> Void

    private let queue: DispatchQueue
    private let restClient: REST.Client
    private let rotationInterval: TimeInterval
    private var completionHandler: CompletionHandler?
    private weak var delegate: RotatePrivateKeyOperationDelegate?
    private var restCancellable: Cancellable?

    init(queue: DispatchQueue,
         restClient: REST.Client,
         rotationInterval: TimeInterval,
         delegate: RotatePrivateKeyOperationDelegate,
         completionHandler: @escaping CompletionHandler)
    {
        self.queue = queue
        self.restClient = restClient
        self.rotationInterval = rotationInterval
        self.delegate = delegate
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(result: nil, error: nil)
                return
            }

            guard let tunnelInfo = self.delegate?.operationDidRequestTunnelInfo(self) else {
                self.finish(result: nil, error: .missingAccount)
                return
            }

            let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
            let timeInterval = Date().timeIntervalSince(creationDate)

            guard timeInterval >= self.rotationInterval else {
                self.finish(result: .throttled(creationDate), error: nil)
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
                        self.delegate?.operation(self, didFinishRotatingPrivateKeyWithNewTunnelSettings: tunnelSettings)
                        self.finish(result: .finished, error: nil)

                    case .failure(let error):
                        self.delegate?.operation(self, didFailToRotatePrivateKeyWithError: error)
                        self.finish(result: nil, error: error)
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

    private func finish(result: TunnelManager.KeyRotationResult?, error: TunnelManager.Error?) {
        completionHandler?(result, error)
        completionHandler = nil

        finish()
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
