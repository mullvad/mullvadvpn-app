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
        retryStrategy: REST.RetryStrategy
    ) async -> Result<NewAccountData, Error>

    func getAccountData(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Account, Error>

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Void, Error>
}

extension REST {
    public final class MullvadAccountProxy: RESTAccountHandling, @unchecked Sendable {
        let transportProvider: APITransportProviderProtocol
        let dispatchQueue: DispatchQueue
        let operationQueue = AsyncOperationQueue()
        let responseDecoder: JSONDecoder

        public init(
            transportProvider: APITransportProviderProtocol,
            dispatchQueue: DispatchQueue,
            responseDecoder: JSONDecoder
        ) {
            self.transportProvider = transportProvider
            self.dispatchQueue = dispatchQueue
            self.responseDecoder = responseDecoder
        }
        public func createAccount(
            retryStrategy: REST.RetryStrategy
        ) async -> Result<NewAccountData, Swift.Error> {
            let request = APIRequest.createAccount(retryStrategy)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: NewAccountData.self,
                    with: responseDecoder
                )
            )

            return await task.startRequest()
        }

        public func getAccountData(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Account, Swift.Error> {
            let request = APIRequest.getAccount(retryStrategy, accountNumber: accountNumber)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: Account.self,
                    with: responseDecoder
                )
            )

            return await task.startRequest()
        }

        public func deleteAccount(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Void, Swift.Error> {
            let request = APIRequest.deleteAccount(retryStrategy, accountNumber: accountNumber)
            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustEmptyResponseHandler()
            )

            return await task.startRequest()
        }
    }
}
