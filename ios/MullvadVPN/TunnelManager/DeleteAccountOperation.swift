//
//  DeleteAccountOperation.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-18.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations

class DeleteAccountOperation: ResultOperation<Void> {
    private let logger = Logger(label: "\(DeleteAccountOperation.self)")

    private let accountNumber: String
    private let accountsProxy: RESTAccountHandling
    private let accessTokenManager: RESTAccessTokenManagement
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        accountsProxy: RESTAccountHandling,
        accessTokenManager: RESTAccessTokenManagement,
        accountNumber: String
    ) {
        self.accountNumber = accountNumber
        self.accountsProxy = accountsProxy
        self.accessTokenManager = accessTokenManager
        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        task = accountsProxy.deleteAccount(
            accountNumber: accountNumber,
            retryStrategy: .default,
            completion: { [weak self] result in
                self?.dispatchQueue.async {
                    switch result {
                    case .success:
                        self?.accessTokenManager.invalidateAllTokens()
                        self?.finish(result: .success(()))
                    case let .failure(error):
                        self?.logger.error(
                            error: error,
                            message: "Failed to delete account."
                        )
                        self?.finish(result: .failure(error))
                    }
                }
            }
        )
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}
