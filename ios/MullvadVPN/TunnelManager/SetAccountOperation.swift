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
    func operation(_ operation: Operation, didFailToSetAccountWithError error: TunnelManager.Error)
    func operationDidSetAccountToken(_ operation: Operation)
}

class SetAccountOperation: AsyncOperation {
    typealias CompletionHandler = (TunnelManager.Error?) -> Void

    private let queue: DispatchQueue
    private let restClient: REST.Client
    private let token: String
    private var completionHandler: CompletionHandler?

    private weak var delegate: SetAccountOperationDelegate?

    init(queue: DispatchQueue, restClient: REST.Client, token: String, delegate: SetAccountOperationDelegate, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.restClient = restClient
        self.token = token
        self.delegate = delegate
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(error: nil)
                return
            }

            switch self.makeTunnelSettings() {
            case .success(let tunnelSettings):
                let interfaceSettings = tunnelSettings.interface

                // Push key if interface addresses were not received yet
                if interfaceSettings.addresses.isEmpty {
                    self.pushWireguardKey(publicKey: interfaceSettings.publicKey) { error in
                        if let error = error {
                            self.delegate?.operation(self, didFailToSetAccountWithError: error)
                            self.finish(error: error)
                        } else {
                            self.delegate?.operationDidSetAccountToken(self)
                            self.finish(error: nil)
                        }
                    }
                } else {
                    self.delegate?.operationDidSetAccountToken(self)
                    self.finish(error: nil)
                }

            case .failure(let error):
                self.delegate?.operation(self, didFailToSetAccountWithError: error)

                self.finish(error: error)
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

    private func finish(error: TunnelManager.Error?) {
        completionHandler?(error)
        completionHandler = nil

        finish()
    }
}
