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
    func operation(_ operation: Operation, didFinishRegeneratingPrivateKeyWithNewTunnelSettings newTunnelSettings: TunnelSettings)
    func operation(_ operation: Operation, didFailToReplacePrivateKeyWithError error: TunnelManager.Error)
}

class RenegeratePrivateKeyOperation: AsyncOperation {
    typealias CompletionHandler = (TunnelManager.Error?) -> Void

    private let queue: DispatchQueue
    private let restClient: REST.Client
    private var completionHandler: CompletionHandler?
    private weak var delegate: RenegeratePrivateKeyOperationDelegate?
    private var restCancellable: Cancellable?

    init(queue: DispatchQueue,
         restClient: REST.Client,
         delegate: RenegeratePrivateKeyOperationDelegate,
         completionHandler: @escaping CompletionHandler)
    {
        self.queue = queue
        self.restClient = restClient
        self.delegate = delegate
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(error: nil)
                return
            }

            guard let tunnelInfo = self.delegate?.operationDidRequestTunnelInfo(self) else {
                self.finish(error: .missingAccount)
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
                        self.delegate?.operation(self, didFinishRegeneratingPrivateKeyWithNewTunnelSettings: tunnelSettings)
                        self.finish(error: nil)

                    case .failure(let error):
                        self.delegate?.operation(self, didFailToReplacePrivateKeyWithError: error)
                        self.finish(error: error)
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

    private func finish(error: TunnelManager.Error?) {
        completionHandler?(error)
        completionHandler = nil

        finish()
    }
}
