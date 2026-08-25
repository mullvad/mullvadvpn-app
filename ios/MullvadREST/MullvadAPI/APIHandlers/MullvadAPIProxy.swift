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

public protocol APIQuerying: Sendable {

    @available(
        *,
        deprecated,
        message: "Use the async version instead."
    )
    func getRelays(
        etag: String?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
    ) -> Cancellable

    func getAddressList(
        retryStrategy: REST.RetryStrategy
    ) async -> Result<[AnyIPEndpoint], Error>

    func getRelays(
        etag: String?,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<REST.ServerRelaysCacheResponse, Error>

    func sendProblemReport(
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Void, Error>

    func initStoreKitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<UUID, Error>

    func checkStoreKitPayment(
        transaction: StoreKitTransaction,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Void, Error>

    func checkApiAvailability(
        retryStrategy: REST.RetryStrategy,
        accessMethod: PersistentAccessMethod
    ) async -> Result<Bool, Error>
}

extension REST {
    public final class MullvadAPIProxy: APIQuerying, @unchecked Sendable {
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

        public func getAddressList(
            retryStrategy: REST.RetryStrategy
        ) async -> Result<[AnyIPEndpoint], Swift.Error> {
            let request = APIRequest.getAddressList(retryStrategy)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: [AnyIPEndpoint].self,
                    with: responseDecoder
                )
            )

            return await task.startRequest()
        }

        public func getRelays(
            etag: String?,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<REST.ServerRelaysCacheResponse, Swift.Error> {

            if var etag {
                // Enforce weak validator to account for some backend caching quirks.
                if etag.starts(with: "\"") {
                    etag.insert(contentsOf: "W/", at: etag.startIndex)
                }
            }
            let responseHandler = rustCustomResponseHandler { data, responseEtag in
                if let responseEtag, responseEtag == etag {
                    return REST.ServerRelaysCacheResponse.notModified
                } else {
                    return REST.ServerRelaysCacheResponse.newContent(responseEtag, data)
                }
            }

            let request = APIRequest.getRelayList(retryStrategy, etag: etag)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: responseHandler
            )

            return await task.startRequest()
        }

        public func sendProblemReport(
            _ body: ProblemReportRequest,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Void, Swift.Error> {
            let request = APIRequest.sendProblemReport(retryStrategy, problemReportRequest: body)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustEmptyResponseHandler()
            )

            return await task.startRequest()
        }

        public func checkStoreKitPayment(
            transaction: StoreKitTransaction,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Void, Swift.Error> {
            let request = APIRequest.checkStorekitPayment(
                retryStrategy: retryStrategy,
                transaction: transaction
            )

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustEmptyResponseHandler()
            )

            return await task.startRequest()
        }

        public func checkApiAvailability(
            retryStrategy: REST.RetryStrategy,
            accessMethod: PersistentAccessMethod
        ) async -> Result<Bool, Swift.Error> {
            let request = APIRequest.checkApiAvailability(retryStrategy, accessMethod: accessMethod)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustEmptyResponseHandler()
            )

            let response = await task.startRequest()
            return switch response {
            case .success:
                .success(true)
            case .failure(let error):
                .failure(error)
            }
        }

        public func initStoreKitPayment(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<UUID, Swift.Error> {
            let request = APIRequest.initStorekitPayment(retryStrategy: retryStrategy, accountNumber: accountNumber)

            struct InitStorekitPaymentResponse: Codable {
                let paymentToken: UUID
            }

            let responseHandler = rustResponseHandler(
                decoding: InitStorekitPaymentResponse.self,
                with: responseDecoder
            )

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: responseHandler
            )

            let response = await task.startRequest()
            return switch response {
            case .success(let value):
                .success(value.paymentToken)
            case .failure(let error):
                .failure(error)
            }
        }

        public func getRelays(
            etag: String?,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
        ) -> Cancellable {
            if var etag {
                // Enforce weak validator to account for some backend caching quirks.
                if etag.starts(with: "\"") {
                    etag.insert(contentsOf: "W/", at: etag.startIndex)
                }
            }

            let responseHandler = rustCustomResponseHandler { data, responseEtag in
                if let responseEtag, responseEtag == etag {
                    return REST.ServerRelaysCacheResponse.notModified
                } else {
                    return REST.ServerRelaysCacheResponse.newContent(responseEtag, data)
                }
            }

            return createNetworkOperation(
                request: .getRelayList(retryStrategy, etag: etag),
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }
        private func createNetworkOperation<Success>(
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

    // MARK: - Response types

    public enum ServerRelaysCacheResponse: Sendable, Decodable {
        case notModified
        case newContent(_ etag: String?, _ rawData: Data)
    }

    public enum CreateApplePaymentResponse: Sendable, Decodable {
        case noTimeAdded(_ expiry: Date)
        case timeAdded(_ timeAdded: Int, _ newExpiry: Date)

        public var newExpiry: Date {
            switch self {
            case let .noTimeAdded(expiry), let .timeAdded(_, expiry):
                return expiry
            }
        }

        public var timeAdded: TimeInterval {
            switch self {
            case .noTimeAdded:
                return 0
            case let .timeAdded(timeAdded, _):
                return TimeInterval(timeAdded)
            }
        }

        /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
        public var formattedTimeAdded: String? {
            let formatter = DateComponentsFormatter()
            formatter.allowedUnits = [.day, .hour]
            formatter.unitsStyle = .full

            return formatter.string(from: self.timeAdded)
        }
    }

    private struct CreateApplePaymentRawResponse: Decodable, Sendable {
        let timeAdded: Int
        let newExpiry: Date
    }
}
