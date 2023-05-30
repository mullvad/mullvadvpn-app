//
//  RESTAPIProxy.swift
//  MullvadREST
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PublicKey

extension REST {
    public final class APIProxy: Proxy<AuthProxyConfiguration> {
        public init(configuration: AuthProxyConfiguration) {
            super.init(
                name: "APIProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/app/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        public func getAddressList(
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<[AnyIPEndpoint]>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                return try self.requestFactory.createRequest(
                    endpoint: endpoint,
                    method: .get,
                    pathTemplate: "api-addrs"
                )
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: [AnyIPEndpoint].self,
                with: responseDecoder
            )

            return addOperation(
                name: "get-api-addrs",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func getRelays(
            etag: String?,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<ServerRelaysCacheResponse>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = try self.requestFactory.createRequestBuilder(
                    endpoint: endpoint,
                    method: .get,
                    pathTemplate: "relays"
                )

                if let etag {
                    requestBuilder.setETagHeader(etag: etag)
                }

                return requestBuilder.getRequest()
            }

            let responseHandler =
                AnyResponseHandler { response, data -> ResponseHandlerResult<ServerRelaysCacheResponse> in
                    let httpStatus = HTTPStatus(rawValue: response.statusCode)

                    switch httpStatus {
                    case let httpStatus where httpStatus.isSuccess:
                        return .decoding {
                            let serverRelays = try self.responseDecoder.decode(
                                ServerRelaysResponse.self,
                                from: data
                            )
                            let newEtag = response.value(forHTTPHeaderField: HTTPHeader.etag)

                            return .newContent(newEtag, serverRelays)
                        }

                    case .notModified where etag != nil:
                        return .success(.notModified)

                    default:
                        return .unhandledResponse(
                            try? self.responseDecoder.decode(
                                ServerErrorResponse.self,
                                from: data
                            )
                        )
                    }
                }

            return addOperation(
                name: "get-relays",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func createApplePayment(
            accountNumber: String,
            receiptString: Data,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<CreateApplePaymentResponse>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var requestBuilder = try self.requestFactory.createRequestBuilder(
                        endpoint: endpoint,
                        method: .post,
                        pathTemplate: "create-apple-payment"
                    )
                    requestBuilder.setAuthorization(authorization)

                    let body = CreateApplePaymentRequest(
                        receiptString: receiptString
                    )
                    try requestBuilder.setHTTPBody(value: body)

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler =
                AnyResponseHandler { response, data -> ResponseHandlerResult<CreateApplePaymentResponse> in
                    if HTTPStatus.isSuccess(response.statusCode) {
                        return .decoding {
                            let serverResponse = try self.responseDecoder.decode(
                                CreateApplePaymentRawResponse.self,
                                from: data
                            )
                            if serverResponse.timeAdded > 0 {
                                return .timeAdded(
                                    serverResponse.timeAdded,
                                    serverResponse.newExpiry
                                )
                            } else {
                                return .noTimeAdded(serverResponse.newExpiry)
                            }
                        }
                    } else {
                        return .unhandledResponse(
                            try? self.responseDecoder.decode(
                                ServerErrorResponse.self,
                                from: data
                            )
                        )
                    }
                }

            return addOperation(
                name: "create-apple-payment",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func sendProblemReport(
            _ body: ProblemReportRequest,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<Void>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = try self.requestFactory.createRequestBuilder(
                    endpoint: endpoint,
                    method: .post,
                    pathTemplate: "problem-report"
                )

                try requestBuilder.setHTTPBody(value: body)

                return requestBuilder.getRequest()
            }

            let responseHandler =
                AnyResponseHandler { response, data -> ResponseHandlerResult<Void> in
                    if HTTPStatus.isSuccess(response.statusCode) {
                        return .success(())
                    } else {
                        return .unhandledResponse(
                            try? self.responseDecoder.decode(
                                ServerErrorResponse.self,
                                from: data
                            )
                        )
                    }
                }

            return addOperation(
                name: "send-problem-report",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        public func submitVoucher(
            voucherCode: String,
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<SubmitVoucherResponse>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var requestBuilder = try self.requestFactory.createRequestBuilder(
                        endpoint: endpoint,
                        method: .post,
                        pathTemplate: "submit-voucher"
                    )

                    requestBuilder.setAuthorization(authorization)

                    try requestBuilder.setHTTPBody(value: SubmitVoucherRequest(voucherCode: voucherCode))

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler = AnyResponseHandler { response, data -> ResponseHandlerResult<SubmitVoucherResponse> in
                if HTTPStatus.isSuccess(response.statusCode) {
                    return .decoding {
                        try self.responseDecoder.decode(SubmitVoucherResponse.self, from: data)
                    }
                } else {
                    return .unhandledResponse(
                        try? self.responseDecoder.decode(ServerErrorResponse.self, from: data)
                    )
                }
            }

            return addOperation(
                name: "send-submit-voucher",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }
    }

    // MARK: - Response types

    public enum ServerRelaysCacheResponse {
        case notModified
        case newContent(_ etag: String?, _ value: ServerRelaysResponse)
    }

    private struct CreateApplePaymentRequest: Encodable {
        let receiptString: Data
    }

    public enum CreateApplePaymentResponse {
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

    private struct CreateApplePaymentRawResponse: Decodable {
        let timeAdded: Int
        let newExpiry: Date
    }

    public struct ProblemReportRequest: Encodable {
        public let address: String
        public let message: String
        public let log: String
        public let metadata: [String: String]

        public init(address: String, message: String, log: String, metadata: [String: String]) {
            self.address = address
            self.message = message
            self.log = log
            self.metadata = metadata
        }
    }

    private struct SubmitVoucherRequest: Encodable {
        let voucherCode: String
    }

    public struct SubmitVoucherResponse: Decodable {
        public let timeAdded: Int
        public let newExpiry: Date

        public var dateComponents: DateComponents {
            return DateComponents(second: timeAdded)
        }
    }
}
