//
//  SetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKit.PublicKey

class SetAccountOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let restClient: REST.Client
    private let accountToken: String
    private var completionHandler: CompletionHandler?

    init(queue: DispatchQueue, state: TunnelManager.State, restClient: REST.Client, accountToken: String, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.restClient = restClient
        self.accountToken = accountToken
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

    private func execute(completionHandler: @escaping CompletionHandler) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        switch makeTunnelSettings() {
        case .success(let tunnelSettings):
            let interfaceSettings = tunnelSettings.interface

            // Push key if interface addresses were not received yet
            if interfaceSettings.addresses.isEmpty {
                pushWireguardKey(publicKey: interfaceSettings.publicKey, completionHandler: completionHandler)
            } else {
                completionHandler(.success(()))
            }

        case .failure(let error):
            completionHandler(.failure(error))
        }
    }

    private func makeTunnelSettings() -> Result<TunnelSettings, TunnelManager.Error> {
        return TunnelSettingsManager.load(searchTerm: .accountToken(self.accountToken))
            .mapError { TunnelManager.Error.readTunnelSettings($0) }
            .map { $0.tunnelSettings }
            .flatMapError { error in
                if case .readTunnelSettings(.lookupEntry(.itemNotFound)) = error {
                    let defaultConfiguration = TunnelSettings()

                    return TunnelSettingsManager
                        .add(configuration: defaultConfiguration, account: self.accountToken)
                        .mapError { .addTunnelSettings($0) }
                        .map { defaultConfiguration }
                } else {
                    return .failure(error)
                }
            }
    }

    private func pushWireguardKey(publicKey: PublicKey, completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void) {
        _ = restClient.pushWireguardKey(token: accountToken, publicKey: publicKey)
            .execute(retryStrategy: .default) { result in
                self.queue.async {
                    self.didPushWireguardKey(result: result, completionHandler: completionHandler)
                }
            }
    }

    private func didPushWireguardKey(result: Result<REST.WireguardAddressesResponse, REST.Error>, completionHandler: @escaping (OperationCompletion<(), TunnelManager.Error>) -> Void) {
        switch result {
        case .success(let associatedAddresses):
            let saveSettingsResult = TunnelSettingsManager.update(searchTerm: .accountToken(accountToken)) { tunnelSettings in
                tunnelSettings.interface.addresses = [
                    associatedAddresses.ipv4Address,
                    associatedAddresses.ipv6Address
                ]
            }

            switch saveSettingsResult {
            case .success(let newTunnelSettings):
                let tunnelInfo = TunnelInfo(
                    token: accountToken,
                    tunnelSettings: newTunnelSettings
                )

                state.tunnelInfo = tunnelInfo

                completionHandler(.success(()))

            case .failure(let error):
                completionHandler(.failure(.updateTunnelSettings(error)))
            }

        case .failure(let error):
            completionHandler(.failure(.pushWireguardKey(error)))
        }
    }
}
