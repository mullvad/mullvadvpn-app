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
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<[AnyIPEndpoint]>
    ) -> Cancellable

    func getRelays(
        signum: (String, Int64)?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
    ) -> Cancellable

    func sendProblemReport(
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<Void>
    ) -> Cancellable

    func submitVoucher(
        voucherCode: String,
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<REST.SubmitVoucherResponse>
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
        accessMethod: PersistentAccessMethod,
        completion: @escaping @Sendable ProxyCompletionHandler<Bool>
    ) -> Cancellable
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
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<[AnyIPEndpoint]>
        ) -> Cancellable {
            let responseHandler = rustResponseHandler(
                decoding: [AnyIPEndpoint].self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .getAddressList(retryStrategy),
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func getRelays(
            signum: (String, Int64)?,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
        ) -> Cancellable {
            let responseHandler = rustCustomResponseHandler { data, digest, timestamp in
                REST.ServerRelaysCacheResponse.newContent(digest, timestamp, data)
            }

            var signumDigest: String? = nil
            var signumTimestamp: Int64? = nil
            if let (digest, timestamp) = signum {
                signumDigest = digest
                signumTimestamp = timestamp
            }
            return createNetworkOperation(
                request: .getRelayList(
                    retryStrategy,
                    digest: signumDigest,
                    timestamp: signumTimestamp),
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func sendProblemReport(
            _ body: ProblemReportRequest,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<Void>
        ) -> Cancellable {
            createNetworkOperation(
                request: .sendProblemReport(retryStrategy, problemReportRequest: body),
                responseHandler: rustEmptyResponseHandler(),
                completionHandler: completionHandler
            )
        }

        public func submitVoucher(
            voucherCode: String,
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<REST.SubmitVoucherResponse>
        ) -> Cancellable {
            AnyCancellable()
        }

        public func checkApiAvailability(
            retryStrategy: REST.RetryStrategy,
            accessMethod: PersistentAccessMethod,
            completion: @escaping @Sendable ProxyCompletionHandler<Bool>
        ) -> Cancellable {
            let responseHandler = rustEmptyResponseHandler()
            return createNetworkOperation(
                request: .checkApiAvailability(retryStrategy, accessMethod: accessMethod),
                responseHandler: responseHandler
            ) { result in
                if case let .failure(err) = result {
                    completion(.failure(err))
                } else {
                    completion(.success(true))
                }
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
