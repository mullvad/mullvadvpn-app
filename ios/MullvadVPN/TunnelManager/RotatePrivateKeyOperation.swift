//
//  RotatePrivateKeyOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class RotatePrivateKeyOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<TunnelManager.KeyRotationResult, TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let restClient: REST.Client
    private let rotationInterval: TimeInterval
    private var completionHandler: CompletionHandler?
    private var restRequest: Cancellable?

    init(queue: DispatchQueue,
         state: TunnelManager.State,
         restClient: REST.Client,
         rotationInterval: TimeInterval,
         completionHandler: @escaping CompletionHandler)
    {
        self.queue = queue
        self.state = state
        self.restClient = restClient
        self.rotationInterval = rotationInterval
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            self.execute { completion in
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
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        guard let tunnelInfo = state.tunnelInfo else {
            completionHandler(.failure(.missingAccount))
            return
        }

        let creationDate = tunnelInfo.tunnelSettings.interface.privateKey.creationDate
        let timeInterval = Date().timeIntervalSince(creationDate)

        guard timeInterval >= rotationInterval else {
            completionHandler(.success(.throttled(creationDate)))
            return
        }

        let newPrivateKey = PrivateKeyWithMetadata()
        let oldPublicKey = tunnelInfo.tunnelSettings.interface.publicKey

        let requestAdapter = self.restClient.replaceWireguardKey(
            token: tunnelInfo.token,
            oldPublicKey: oldPublicKey,
            newPublicKey: newPrivateKey.publicKey
        )

        restRequest = requestAdapter.execute(retryStrategy: .default) { result in
            self.queue.async {
                self.didRotatePrivateKey(
                    result: result,
                    accountToken: tunnelInfo.token,
                    newPrivateKey: newPrivateKey,
                    completionHandler: completionHandler
                )
            }
        }
    }

    private func didRotatePrivateKey(result: Result<REST.WireguardAddressesResponse, REST.Error>, accountToken: String, newPrivateKey: PrivateKeyWithMetadata, completionHandler: @escaping CompletionHandler) {
        let saveResult = Self.handleResponse(accountToken: accountToken, newPrivateKey: newPrivateKey, result: result)

        switch saveResult {
        case .success(let tunnelSettings):
            state.tunnelInfo?.tunnelSettings = tunnelSettings

            completionHandler(.success(.finished))

        case .failure(let error):
            completionHandler(.failure(error))
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
