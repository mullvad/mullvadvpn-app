//
//  RegeneratePrivateKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class RegeneratePrivateKeyOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let restClient: REST.Client
    private var completionHandler: CompletionHandler?
    private var restRequest: Cancellable?

    init(queue: DispatchQueue, state: TunnelManager.State, restClient: REST.Client, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.restClient = restClient
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            self.execute { [weak self] completion in
                guard let self = self else { return }

                self.completionHandler?(completion)
                self.completionHandler = nil

                self.finish()
            }
        }
    }

    override func cancel() {
        super.cancel()

        queue.async {
            self.restRequest?.cancel()
        }
    }

    private func execute(completionHandler: @escaping CompletionHandler) {
        guard !self.isCancelled else {
            completionHandler(.cancelled)
            return
        }

        guard let tunnelInfo = state.tunnelInfo else {
            completionHandler(.failure(.missingAccount))
            return
        }

        let newPrivateKey = PrivateKeyWithMetadata()
        let oldPublicKey = tunnelInfo.tunnelSettings.interface.publicKey

        let restRequestAdapter = self.restClient.replaceWireguardKey(
            token: tunnelInfo.token,
            oldPublicKey: oldPublicKey,
            newPublicKey: newPrivateKey.publicKey
        )

        restRequest = restRequestAdapter.execute(retryStrategy: .default) { restResult in
            self.queue.async {
                let saveResult = Self.handleResponse(accountToken: tunnelInfo.token, newPrivateKey: newPrivateKey, result: restResult)

                if case .success(let newTunnelSettings) = saveResult {
                    self.state.tunnelInfo?.tunnelSettings = newTunnelSettings
                }

                completionHandler(OperationCompletion(result: saveResult.map { _ in () }))
            }
        }
    }

    private class func handleResponse(accountToken: String, newPrivateKey: PrivateKeyWithMetadata, result: Result<REST.WireguardAddressesResponse, REST.Error>) -> Result<TunnelSettings, TunnelManager.Error> {
        return result.flatMapError { restError in
            return .failure(.replaceWireguardKey(restError))
        }
        .flatMap { associatedAddresses in
            return TunnelSettingsManager.update(searchTerm: .accountToken(accountToken)) { newTunnelSettings in
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
