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
                            self.delegate?.operation(self, didFinishRotatingPrivateKeyWithNewTunnelSettings: newTunnelSettings)
                            self.finish(result: .finished, error: nil)

                        case .failure(let error):
                            let tunnelManagerError = TunnelManager.Error.updateTunnelSettings(error)

                            self.delegate?.operation(self, didFailToRotatePrivateKeyWithError: tunnelManagerError)
                            self.finish(result: nil, error: tunnelManagerError)
                        }

                    case .failure(let error):
                        let tunnelManagerError = TunnelManager.Error.replaceWireguardKey(error)

                        self.delegate?.operation(self, didFailToRotatePrivateKeyWithError: tunnelManagerError)
                        self.finish(result: nil, error: tunnelManagerError)
                    }
                }
            }
        }
    }

    private func finish(result: TunnelManager.KeyRotationResult?, error: TunnelManager.Error?) {
        completionHandler?(result, error)
        completionHandler = nil

        finish()
    }
}
