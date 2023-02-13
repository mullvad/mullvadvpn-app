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

extension REST {
    public final class AccessTokenManager {
        private let logger = Logger(label: "REST.AccessTokenManager")
        private let operationQueue = AsyncOperationQueue.makeSerial()
        private let dispatchQueue = DispatchQueue(label: "REST.AccessTokenManager.dispatchQueue")
        private let proxy: AuthenticationProxy
        private var tokens = [String: AccessTokenData]()

        public init(authenticationProxy: AuthenticationProxy) {
            proxy = authenticationProxy
        }

        func getAccessToken(
            accountNumber: String,
            completionHandler: @escaping (Result<REST.AccessTokenData, Swift.Error>) -> Void
        ) -> Cancellable {
            let operation = ResultBlockOperation<REST.AccessTokenData>(dispatchQueue: dispatchQueue)

            operation.setExecutionBlock { operation in
                if let tokenData = self.tokens[accountNumber], tokenData.expiry > Date() {
                    operation.finish(result: .success(tokenData))
                    return
                }

                let task = self.proxy.getAccessToken(
                    accountNumber: accountNumber,
                    retryStrategy: .noRetry
                ) { result in
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

                        operation.finish(result: result)
                    }
                }

                operation.addCancellationBlock {
                    task.cancel()
                }
            }

            operation.completionQueue = .main
            operation.completionHandler = completionHandler

            operationQueue.addOperation(operation)

            return operation
        }
    }
}
