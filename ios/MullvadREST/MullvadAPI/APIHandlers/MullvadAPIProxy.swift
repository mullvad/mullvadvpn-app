//
//  MullvadAPIProxy.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-03-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes
import Operations
import WireGuardKitTypes

public protocol APIQuerying: Sendable {
    func getAddressList(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<[AnyIPEndpoint]>
    ) -> Cancellable

    func getRelays(
        etag: String?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
    ) -> Cancellable

    func createApplePayment(
        accountNumber: String,
        receiptString: Data
    ) -> any RESTRequestExecutor<REST.CreateApplePaymentResponse>

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

    func initStorekitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<String>
    ) -> Cancellable

    func checkStorekitPayment(
        accountNumber: String,
        transaction: StorekitTransaction,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping @Sendable ProxyCompletionHandler<Void>
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
            completionHandler: @escaping @Sendable ProxyCompletionHandler<[AnyIPEndpoint]>
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
            etag: String?,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping @Sendable ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
        ) -> Cancellable {
            if var etag {
                // Enforce weak validator to account for some backend caching quirks.
                if etag.starts(with: "\"") {
                    etag.insert(contentsOf: "W/", at: etag.startIndex)
                }
            }

            let responseHandler = rustCustomResponseHandler { [weak self] data, responseEtag in
                if let responseEtag, responseEtag == etag {
                    return REST.ServerRelaysCacheResponse.notModified
                } else {
                    // Discarding result since we're only interested in knowing that it's parseable.
                    let canDecodeResponse = (try? self?.responseDecoder.decode(
                        REST.ServerRelaysResponse.self,
                        from: data
                    )) != nil

                    return canDecodeResponse ? REST.ServerRelaysCacheResponse.newContent(responseEtag, data) : nil
                }
            }

            return createNetworkOperation(
                request: .getRelayList(retryStrategy, etag: etag),
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func createApplePayment(
            accountNumber: String,
            receiptString: Data
        ) -> any RESTRequestExecutor<REST.CreateApplePaymentResponse> {
            RESTRequestExecutorStub<REST.CreateApplePaymentResponse>(success: {
                .timeAdded(42, .distantFuture)
            })
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

        public func initStorekitPayment(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<String>
        ) -> any MullvadTypes.Cancellable {
            struct InitStorekitPaymentResponse: Codable {
                let paymentToken: String
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

        public func checkStorekitPayment(
            accountNumber: String,
            transaction: StorekitTransaction,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<Void>
        ) -> any MullvadTypes.Cancellable {
            let responseHandler = rustEmptyResponseHandler()

            return createNetworkOperation(
                request:
                .checkStorekitPayment(
                    retryStrategy: retryStrategy,
                    accountNumber: accountNumber,
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
        case newContent(_ etag: String?, _ rawData: Data)
    }

    private struct CreateApplePaymentRequest: Encodable, Sendable {
        let receiptString: Data
    }

    public enum CreateApplePaymentResponse: Sendable {
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

// TODO: Remove when "createApplePayment" func is implemented.
private struct RESTRequestExecutorStub<Success: Sendable>: RESTRequestExecutor {
    var success: (() -> Success)?

    func execute(completionHandler: @escaping (Result<Success, Error>) -> Void) -> Cancellable {
        if let result = success?() {
            completionHandler(.success(result))
        }
        return AnyCancellable()
    }

    func execute(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping (Result<Success, Error>) -> Void
    ) -> Cancellable {
        if let result = success?() {
            completionHandler(.success(result))
        }
        return AnyCancellable()
    }

    func execute() async throws -> Success {
        try await execute(retryStrategy: .noRetry)
    }

    func execute(retryStrategy: REST.RetryStrategy) async throws -> Success {
        guard let success = success else { throw POSIXError(.EINVAL) }

        return success()
    }
}
