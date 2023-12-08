//
//  RESTAccessTokenManager.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Operations

public protocol RESTAccessTokenManagement {
    func getAccessToken(
        accountNumber: String,
        completionHandler: @escaping ProxyCompletionHandler<REST.AccessTokenData>
    ) -> Cancellable

    func invalidateAllTokens()
}

extension REST {
    public final class AccessTokenManager: RESTAccessTokenManagement {
        private let logger = Logger(label: "REST.AccessTokenManager")
        private let operationQueue = AsyncOperationQueue.makeSerial()
        private let dispatchQueue = DispatchQueue(label: "REST.AccessTokenManager.dispatchQueue")
        private let proxy: AuthenticationProxy
        private var tokens = [String: AccessTokenData]()

        public init(authenticationProxy: AuthenticationProxy) {
            proxy = authenticationProxy
        }

        public func getAccessToken(
            accountNumber: String,
            completionHandler: @escaping ProxyCompletionHandler<REST.AccessTokenData>
        ) -> Cancellable {
            let operation =
                ResultBlockOperation<REST.AccessTokenData>(dispatchQueue: dispatchQueue) { finish -> Cancellable in
                    if let tokenData = self.tokens[accountNumber], tokenData.expiry > Date() {
                        finish(.success(tokenData))
                        return AnyCancellable()
                    }

                    return self.proxy.getAccessToken(accountNumber: accountNumber, retryStrategy: .noRetry) { result in
                        self.dispatchQueue.async {
                            switch result {
                            case let .success(tokenData):
                                self.tokens[accountNumber] = tokenData

                            case let .failure(error) where !error.isOperationCancellationError:
                                self.logger.error(
                                    error: error,
                                    message: "Failed to fetch access token."
                                )

                            default:
                                break
                            }

                            finish(result)
                        }
                    }
                }

            operation.completionQueue = .main
            operation.completionHandler = completionHandler

            operationQueue.addOperation(operation)

            return operation
        }

        public func invalidateAllTokens() {
            operationQueue.addOperation(AsyncBlockOperation(dispatchQueue: dispatchQueue) { [weak self] in
                guard let self else {
                    return
                }
                self.tokens.removeAll()
            })
        }
    }
}
