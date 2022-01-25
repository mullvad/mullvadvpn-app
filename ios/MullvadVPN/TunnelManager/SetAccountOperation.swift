//
//  SetAccountOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 16/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKit.PublicKey

protocol SetAccountOperationDelegate: AnyObject {
    func operation(_ operation: Operation, didSetTunnelInfo newTunnelInfo: TunnelInfo)
    func operation(_ operation: Operation, didFinishSettingAccountTokenWithCompletion completion: OperationCompletion<(), TunnelManager.Error>)
}

class SetAccountOperation: BaseTunnelOperation<(), TunnelManager.Error> {
    private let restClient: REST.Client
    private let token: String

    private weak var delegate: SetAccountOperationDelegate?

    init(queue: DispatchQueue, restClient: REST.Client, token: String, delegate: SetAccountOperationDelegate) {
        self.restClient = restClient
        self.token = token
        self.delegate = delegate

        super.init(queue: queue)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.completeOperation(completion: .cancelled)
                return
            }

            switch self.makeTunnelSettings() {
            case .success(let tunnelSettings):
                let interfaceSettings = tunnelSettings.interface

                // Push key if interface addresses were not received yet
                if interfaceSettings.addresses.isEmpty {
                    self.pushWireguardKey(publicKey: interfaceSettings.publicKey) { error in
                        let completion: OperationCompletion<(), TunnelManager.Error> = error.map { .failure($0) } ?? .success(())

                        self.completeOperation(completion: completion)
                    }
                } else {
                    self.completeOperation(completion: .success(()))
                }

            case .failure(let error):
                self.completeOperation(completion: .failure(error))
            }
        }
    }

    private func makeTunnelSettings() -> Result<TunnelSettings, TunnelManager.Error> {
        return TunnelSettingsManager.load(searchTerm: .accountToken(self.token))
            .mapError { TunnelManager.Error.readTunnelSettings($0) }
            .map { $0.tunnelSettings }
            .flatMapError { error in
                if case .readTunnelSettings(.lookupEntry(.itemNotFound)) = error {
                    let defaultConfiguration = TunnelSettings()

                    return TunnelSettingsManager
                        .add(configuration: defaultConfiguration, account: self.token)
                        .mapError { .addTunnelSettings($0) }
                        .map { defaultConfiguration }
                } else {
                    return .failure(error)
                }
            }
    }

    private func pushWireguardKey(publicKey: PublicKey, completionHandler: @escaping (TunnelManager.Error?) -> Void) {
        _ = restClient.pushWireguardKey(token: token, publicKey: publicKey)
            .execute { result in
                self.queue.async {
                    switch result {
                    case .success(let associatedAddresses):
                        let saveSettingsResult = TunnelSettingsManager.update(searchTerm: .accountToken(self.token)) { tunnelSettings in
                            tunnelSettings.interface.addresses = [
                                associatedAddresses.ipv4Address,
                                associatedAddresses.ipv6Address
                            ]
                        }

                        switch saveSettingsResult {
                        case .success(let newTunnelSettings):
                            let tunnelInfo = TunnelInfo(
                                token: self.token,
                                tunnelSettings: newTunnelSettings
                            )

                            self.delegate?.operation(self, didSetTunnelInfo: tunnelInfo)

                            completionHandler(nil)

                        case .failure(let error):
                            completionHandler(.updateTunnelSettings(error))
                        }

                    case .failure(let error):
                        completionHandler(.pushWireguardKey(error))
                    }
                }
            }
    }

    override func completeOperation(completion: OperationCompletion<(), TunnelManager.Error>) {
        delegate?.operation(self, didFinishSettingAccountTokenWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }
}
