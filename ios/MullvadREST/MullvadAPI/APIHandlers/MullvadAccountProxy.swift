// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadRustRuntime
import MullvadTypes
import Operations

public protocol RESTAccountHandling: Sendable {
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<NewAccountData>
    ) -> Cancellable

    func getAccountData(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Account, Error>

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable
}

extension REST {
    public final class MullvadAccountProxy: RESTAccountHandling, @unchecked Sendable {
        let apiContext: ApiContext
        let transportProvider: APITransportProviderProtocol
        let dispatchQueue: DispatchQueue
        let operationQueue = AsyncOperationQueue()
        let responseDecoder: JSONDecoder

        public init(
            apiContext: ApiContext,
            transportProvider: APITransportProviderProtocol,
            dispatchQueue: DispatchQueue,
            responseDecoder: JSONDecoder
        ) {
            self.apiContext = apiContext
            self.transportProvider = transportProvider
            self.dispatchQueue = dispatchQueue
            self.responseDecoder = responseDecoder
        }

        public func createAccount(
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<NewAccountData>
        ) -> Cancellable {
            Task {
                do {
                    let response = try await apiContext.createAccount(retryStrategy: retryStrategy.toRustStrategy())
                    let body = try response.get().body
                    let account = try responseDecoder.decode(NewAccountData.self, from: body)
                    completion(.success(account))
                } catch {
                    completion(.failure(error))
                }
            }.cancellable
        }

        public func getAccountData(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Account, Swift.Error> {
            do {
                let response = try await apiContext.getAccount(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber)

                let data = try response.get().body
                let account = try responseDecoder.decode(Account.self, from: data)
                return .success(account)
            } catch {
                return .failure(error)
            }
        }

        public func deleteAccount(
            accountNumber: String,
            retryStrategy: RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Void>
        ) -> Cancellable {
            let request = APIRequest.deleteAccount(retryStrategy, accountNumber: accountNumber)

            let networkOperation = MullvadApiNetworkOperation(
                name: request.name,
                dispatchQueue: dispatchQueue,
                request: request,
                transportProvider: transportProvider,
                responseDecoder: responseDecoder,
                responseHandler: rustEmptyResponseHandler(),
                completionHandler: completion
            )

            operationQueue.addOperation(networkOperation)

            return networkOperation
        }

        private func createNetworkOperation<Success: Decodable>(
            request: APIRequest,
            responseHandler: RustResponseHandler<Success>,
            completionHandler: @escaping @Sendable ProxyCompletionHandler<Success>
        ) -> MullvadApiNetworkOperation<Success> {
            let networkOperation = MullvadApiNetworkOperation(
                name: request.name,
                dispatchQueue: dispatchQueue,
                request: request,
                transportProvider: transportProvider,
                responseDecoder: responseDecoder,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(networkOperation)

            return networkOperation
        }
    }
}
