//
//  RESTAPIProxy.swift
//  MullvadREST
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes
import Operations
import WireGuardKitTypes

extension REST {
    public final class APIProxy: Proxy<AuthProxyConfiguration>, APIQuerying, @unchecked Sendable {
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
            completionHandler: @escaping @Sendable ProxyCompletionHandler<[AnyIPEndpoint]>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                try self.requestFactory.createRequest(
                    endpoint: endpoint,
                    method: .get,
                    pathTemplate: "api-addrs"
                )
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: [AnyIPEndpoint].self,
                with: responseDecoder
            )

            let executor = makeRequestExecutor(
                name: "get-api-addrs",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completionHandler)
        }

        public func getRelays(
            etag: String?,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping @Sendable ProxyCompletionHandler<ServerRelaysCacheResponse>
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
                            // Discarding result since we're only interested in knowing that it's parseable.
                            _ = try self.responseDecoder.decode(
                                ServerRelaysResponse.self,
                                from: data
                            )
                            let newEtag = response.value(forHTTPHeaderField: HTTPHeader.etag)

                            return .newContent(newEtag, data)
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

            let executor = makeRequestExecutor(
                name: "get-relays",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completionHandler)
        }

        public func createApplePayment(
            accountNumber: String,
            receiptString: Data
        ) -> any RESTRequestExecutor<CreateApplePaymentResponse> {
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

            return makeRequestExecutor(
                name: "create-apple-payment",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )
        }

        public func sendProblemReport(
            _ body: ProblemReportRequest,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<Void>
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

            let executor = makeRequestExecutor(
                name: "send-problem-report",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completionHandler)
        }

        public func submitVoucher(
            voucherCode: String,
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping @Sendable ProxyCompletionHandler<SubmitVoucherResponse>
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

            let executor = makeRequestExecutor(
                name: "submit-voucher",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completionHandler)
        }
    }

    // MARK: - Response types

    private struct CreateApplePaymentRequest: Encodable, Sendable {
        let receiptString: Data
    }

    private struct CreateApplePaymentRawResponse: Decodable, Sendable {
        let timeAdded: Int
        let newExpiry: Date
    }

    private struct SubmitVoucherRequest: Encodable, Sendable {
        let voucherCode: String
    }

    public struct SubmitVoucherResponse: Decodable, Sendable {
        public let timeAdded: Int
        public let newExpiry: Date

        public var dateComponents: DateComponents {
            DateComponents(second: timeAdded)
        }
    }
}
