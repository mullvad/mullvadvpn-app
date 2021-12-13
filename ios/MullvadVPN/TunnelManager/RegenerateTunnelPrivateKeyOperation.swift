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

            _ = self.restClient.replaceWireguardKey(
                token: tunnelInfo.token,
                oldPublicKey: oldPublicKey,
                newPublicKey: newPrivateKey.publicKey
            ).execute { result in
                self.queue.async {
                    switch result {
                    case .success(let associatedAddresses):
                        let saveResult = TunnelSettingsManager.update(searchTerm: .accountToken(tunnelInfo.token)) { newTunnelSettings in
                            newTunnelSettings.interface.privateKey = newPrivateKey
                            newTunnelSettings.interface.addresses = [
                                associatedAddresses.ipv4Address,
                                associatedAddresses.ipv6Address
                            ]
                        }

                        switch saveResult {
                        case .success(let newTunnelSettings):
                            self.delegate?.operation(self, didFinishRegeneratingPrivateKeyWithNewTunnelSettings: newTunnelSettings)
                            self.finish(error: nil)

                        case .failure(let error):
                            let tunnelManagerError = TunnelManager.Error.updateTunnelSettings(error)

                            self.delegate?.operation(self, didFailToReplacePrivateKeyWithError: tunnelManagerError)
                            self.finish(error: tunnelManagerError)
                        }

                    case .failure(let error):
                        let tunnelManagerError = TunnelManager.Error.replaceWireguardKey(error)

                        self.delegate?.operation(self, didFailToReplacePrivateKeyWithError: tunnelManagerError)
                        self.finish(error: tunnelManagerError)
                    }
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
