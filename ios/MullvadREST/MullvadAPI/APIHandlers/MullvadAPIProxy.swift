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
    func getAddressList(
        retryStrategy: REST.RetryStrategy
    ) async throws -> [AnyIPEndpoint]

    func getRelays(
        sigsum: (String, Int64)?,
        retryStrategy: REST.RetryStrategy,
    ) async throws -> REST.ServerRelaysCacheResponse?

    func sendProblemReport(
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<Void>
    ) -> Cancellable

    func initStoreKitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<UUID>
    ) -> Cancellable

    func checkStoreKitPayment(
        transaction: StoreKitTransaction,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<Void>
    ) -> Cancellable

    func checkApiAvailability(
        retryStrategy: REST.RetryStrategy,
        accessMethod: PersistentAccessMethod
    ) async throws -> Bool
}

extension REST {
    public final class MullvadAPIProxy: APIQuerying, @unchecked Sendable {
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

        public func getAddressList(
            retryStrategy: REST.RetryStrategy
        ) async throws -> [AnyIPEndpoint] {
            let response = try await apiContext.getAddresses(retryStrategy: retryStrategy.toRustStrategy())
            return try decode(decoding: [AnyIPEndpoint].self, responseDecoder: responseDecoder, response: response)
        }

        public func getRelays(
            sigsum: (String, Int64)?,
            retryStrategy: REST.RetryStrategy,
        ) async throws -> REST.ServerRelaysCacheResponse? {
            // An unchanged sigsum digest yields an empty body.
            let responseHandler = rustCustomResponseHandler { data, digest, timestamp in
                data.isEmpty
                    ? REST.ServerRelaysCacheResponse.notModified
                    : REST.ServerRelaysCacheResponse.newContent(digest, timestamp, data)
            }

            var sigsumDigest: String? = nil
            var sigsumTimestamp: Int64? = nil
            if let (digest, timestamp) = sigsum {
                sigsumDigest = digest
                sigsumTimestamp = timestamp
            }
            let response = try await apiContext.getRelays(
                retryStrategy: retryStrategy.toRustStrategy(),
                digest: sigsumDigest,
                digestTimestamp: sigsumTimestamp)
            switch response {
            case .body(let body, let statusCode, let sigsumDigest, let sigsumTimestamp):
                if body.isEmpty {
                    return REST.ServerRelaysCacheResponse.notModified
                } else {
                    return REST.ServerRelaysCacheResponse.newContent(sigsumDigest, sigsumTimestamp, body)
                }
            case .error(let statusCode, let errorDescription, let serverResponseCode):
                throw APIError(
                    statusCode: Int(statusCode),
                    errorDescription: errorDescription ?? "",
                    serverResponseCode: serverResponseCode)
            }
        }

        public func sendProblemReport(
            _ body: ProblemReportRequest,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<Void>
        ) -> Cancellable {
            Task {
                do {
                    let response = try await apiContext.sendProblemReport(
                        retryStrategy: retryStrategy.toRustStrategy(),
                        problemReportRequest: body)
                    switch response {
                    case .body:
                        completionHandler(.success(()))
                    case .error(let statusCode, let errorDescription, let serverResponseCode):
                        completionHandler(
                            .failure(
                                APIError(
                                    statusCode: Int(statusCode),
                                    errorDescription: errorDescription ?? "",
                                    serverResponseCode: serverResponseCode)))
                    }
                } catch {
                    completionHandler(.failure(error))
                }
            }.cancellable
        }

        public func checkApiAvailability(
            retryStrategy: REST.RetryStrategy,
            accessMethod: PersistentAccessMethod,
        ) async throws -> Bool {
            let responseHandler = rustEmptyResponseHandler()
            let response = try await apiContext.apiAddrsAvailable(
                retryStrategy: retryStrategy.toRustStrategy(), accessMethodSetting: convertAccessMethod(accessMethod: accessMethod)!)
            if case .error = response {
                return false
            } else {
                return true
            }
        }

        public func initStoreKitPayment(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<UUID>
        ) -> Cancellable {
            struct InitStorekitPaymentResponse: Codable {
                let paymentToken: UUID
            }

            let responseHandler = rustResponseHandler(
                decoding: InitStorekitPaymentResponse.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request:
                    .initStorekitPayment(retryStrategy: retryStrategy, accountNumber: accountNumber),
                responseHandler: responseHandler,
                completionHandler: { completionHandler($0.map { $0.paymentToken }) }
            )
        }

        public func checkStoreKitPayment(
            transaction: StoreKitTransaction,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<Void>
        ) -> Cancellable {
            let responseHandler = rustEmptyResponseHandler()

            return createNetworkOperation(
                request:
                    .checkStorekitPayment(
                        retryStrategy: retryStrategy,
                        transaction: transaction
                    ),
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
        case newContent(_ digest: String?, _ timestamp: Int64?, _ rawData: Data)
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
