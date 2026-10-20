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
    ) async throws -> Account

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
    ) async throws
}

extension REST {
    public final class MullvadAccountProxy: RESTAccountHandling, @unchecked Sendable {
        let transportProvider: APITransportProviderProtocol
        let dispatchQueue: DispatchQueue
        let operationQueue = AsyncOperationQueue()
        let responseDecoder: JSONDecoder
        let apiContext: ApiContext

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
            let responseHandler = rustResponseHandler(
                decoding: NewAccountData.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .createAccount(retryStrategy),
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        public func getAccountData(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async throws -> Account {
            let response = try await apiContext.getAccount(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber)

            return try decode(
                decoding: Account.self,
                responseDecoder: responseDecoder,
                response: response)
        }

        public func deleteAccount(
            accountNumber: String,
            retryStrategy: RetryStrategy,
        ) async throws {
            let response = try await apiContext.deleteAccount(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber)

            if case let .error(statusCode, errorDescription, serverResponseCode) = response {
                throw APIError(statusCode: Int(statusCode), errorDescription: errorDescription ?? "", serverResponseCode: serverResponseCode)
            }
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
