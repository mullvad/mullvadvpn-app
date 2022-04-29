//
//  RESTAPIProxy.swift
//  MullvadVPN
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import class WireGuardKitTypes.PublicKey
import struct WireGuardKitTypes.IPAddressRange

extension REST {
    class APIProxy: Proxy<ProxyConfiguration> {
        init(configuration: ProxyConfiguration) {
            super.init(
                name: "APIProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/app/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: ResponseDecoder(
                    decoder: Coding.makeJSONDecoder()
                )
            )
        }

        func createAccount(
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<AccountResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                let request = self.requestFactory.createURLRequest(
                    endpoint: endpoint,
                    method: .post,
                    path: "accounts"
                )

                return .success(request)
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: AccountResponse.self,
                with: responseDecoder
            )

            return addOperation(
                name: "create-account",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        func getAddressList(
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<[AnyIPEndpoint]>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                let request = self.requestFactory.createURLRequest(
                    endpoint: endpoint,
                    method: .get,
                    path: "api-addrs"
                )

                return .success(request)
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

        func getRelays(
            etag: String?,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<ServerRelaysCacheResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = self.requestFactory.createURLRequestBuilder(
                    endpoint: endpoint,
                    method: .get,
                    path: "relays"
                )

                if let etag = etag {
                    requestBuilder.setETagHeader(etag: etag)
                }

                return .success(requestBuilder.getURLRequest())
            }

            let responseHandler = AnyResponseHandler { response, data -> Result<ServerRelaysCacheResponse, REST.Error> in
                if HTTPStatus.isSuccess(response.statusCode) {
                    return self.responseDecoder.decodeSuccessResponse(ServerRelaysResponse.self, from: data)
                        .map { serverRelays in
                            let newEtag = response.value(forCaseInsensitiveHTTPHeaderField: HTTPHeader.etag)
                            return .newContent(newEtag, serverRelays)
                        }
                } else if response.statusCode == HTTPStatus.notModified && etag != nil {
                    return .success(.notModified)
                } else {
                    return self.responseDecoder.decodeErrorResponseAndMapToServerError(from: data)
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

        func getAccountExpiry(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<AccountResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = self.requestFactory
                    .createURLRequestBuilder(
                        endpoint: endpoint,
                        method: .get,
                        path: "me"
                    )
                requestBuilder.setAuthorization(.accountNumber(accountNumber))

                return .success(requestBuilder.getURLRequest())
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: AccountResponse.self,
                with: responseDecoder
            )

            return addOperation(
                name: "get-account-expiry",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        func getWireguardKey(
            accountNumber: String,
            publicKey: PublicKey,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<WireguardAddressesResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                let urlEncodedPublicKey = publicKey.base64Key
                    .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!
                let path = "wireguard-keys/".appending(urlEncodedPublicKey)

                var requestBuilder = self.requestFactory
                    .createURLRequestBuilder(
                        endpoint: endpoint,
                        method: .get,
                        path: path
                    )
                requestBuilder.setAuthorization(.accountNumber(accountNumber))

                return .success(requestBuilder.getURLRequest())
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: WireguardAddressesResponse.self,
                with: responseDecoder
            )

            return addOperation(
                name: "get-wireguard-key",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        func pushWireguardKey(
            accountNumber: String,
            publicKey: PublicKey,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<WireguardAddressesResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = self.requestFactory.createURLRequestBuilder(
                    endpoint: endpoint,
                    method: .post,
                    path: "wireguard-keys"
                )
                requestBuilder.setAuthorization(.accountNumber(accountNumber))

                return Result {
                    let body = PushWireguardKeyRequest(
                        pubkey: publicKey.rawValue
                    )
                    try requestBuilder.setHTTPBody(value: body)
                }
                .mapError { error in
                    return .encodePayload(error)
                }
                .map { _ in
                    return requestBuilder.getURLRequest()
                }
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: WireguardAddressesResponse.self,
                with: responseDecoder
            )

            return addOperation(
                name: "push-wireguard-key",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        func replaceWireguardKey(
            accountNumber: String,
            oldPublicKey: PublicKey,
            newPublicKey: PublicKey,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<WireguardAddressesResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = self.requestFactory.createURLRequestBuilder(
                    endpoint: endpoint,
                    method: .post,
                    path: "replace-wireguard-key"
                )
                requestBuilder.setAuthorization(.accountNumber(accountNumber))

                return Result {
                    let body = ReplaceWireguardKeyRequest(
                        old: oldPublicKey.rawValue,
                        new: newPublicKey.rawValue
                    )
                    try requestBuilder.setHTTPBody(value: body)
                }
                .mapError { error in
                    return .encodePayload(error)
                }
                .map { _ in
                    return requestBuilder.getURLRequest()
                }
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: WireguardAddressesResponse.self,
                with: responseDecoder
            )

            return addOperation(
                name: "replace-wireguard-key",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        func deleteWireguardKey(
            accountNumber: String,
            publicKey: PublicKey,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<Void>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                let urlEncodedPublicKey = publicKey.base64Key
                    .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

                let path = "wireguard-keys/".appending(urlEncodedPublicKey)
                var requestBuilder = self.requestFactory
                    .createURLRequestBuilder(
                        endpoint: endpoint,
                        method: .delete,
                        path: path
                    )
                requestBuilder.setAuthorization(.accountNumber(accountNumber))

                return .success(requestBuilder.getURLRequest())
            }

            let responseHandler = AnyResponseHandler { response, data -> Result<Void, REST.Error> in
                if HTTPStatus.isSuccess(response.statusCode) {
                    return .success(())
                } else {
                    return self.responseDecoder.decodeErrorResponseAndMapToServerError(from: data)
                }
            }

            return addOperation(
                name: "delete-wireguard-key",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }

        func createApplePayment(
            accountNumber: String,
            receiptString: Data,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<CreateApplePaymentResponse>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = self.requestFactory
                    .createURLRequestBuilder(
                        endpoint: endpoint,
                        method: .post,
                        path: "create-apple-payment"
                    )
                requestBuilder.setAuthorization(.accountNumber(accountNumber))

                return Result {
                    let body = CreateApplePaymentRequest(
                        receiptString: receiptString
                    )
                    try requestBuilder.setHTTPBody(value: body)
                }
                .mapError { error in
                    return .encodePayload(error)
                }
                .map { _ in
                    return requestBuilder.getURLRequest()
                }
            }

            let responseHandler = AnyResponseHandler { response, data -> Result<CreateApplePaymentResponse, REST.Error> in
                if HTTPStatus.isSuccess(response.statusCode) {
                    return self.responseDecoder.decodeSuccessResponse(CreateApplePaymentRawResponse.self, from: data)
                        .map { (response) in
                            if response.timeAdded > 0 {
                                return .timeAdded(response.timeAdded, response.newExpiry)
                            } else {
                                return .noTimeAdded(response.newExpiry)
                            }
                        }
                } else {
                    return self.responseDecoder.decodeErrorResponseAndMapToServerError(from: data)
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

        func sendProblemReport(
            _ body: ProblemReportRequest,
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping CompletionHandler<Void>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = self.requestFactory.createURLRequestBuilder(
                    endpoint: endpoint,
                    method: .post,
                    path: "problem-report"
                )

                return Result {
                    try requestBuilder.setHTTPBody(value: body)
                }
                .mapError { error in
                    return .encodePayload(error)
                }
                .map { _ in
                    return requestBuilder.getURLRequest()
                }
            }

            let responseHandler = AnyResponseHandler { response, data -> Result<Void, REST.Error> in
                if HTTPStatus.isSuccess(response.statusCode) {
                    return .success(())
                } else {
                    return self.responseDecoder.decodeErrorResponseAndMapToServerError(from: data)
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
    }

    // MARK: - Response types

    struct AccountResponse: Decodable {
        let token: String
        let expires: Date
    }

    enum ServerRelaysCacheResponse {
        case notModified
        case newContent(_ etag: String?, _ value: ServerRelaysResponse)
    }

    struct WireguardAddressesResponse: Decodable {
        let id: String
        let pubkey: Data
        let ipv4Address: IPAddressRange
        let ipv6Address: IPAddressRange
    }

    fileprivate struct PushWireguardKeyRequest: Encodable {
        let pubkey: Data
    }

    fileprivate struct ReplaceWireguardKeyRequest: Encodable {
        let old: Data
        let new: Data
    }

    fileprivate struct CreateApplePaymentRequest: Encodable {
        let receiptString: Data
    }

    enum CreateApplePaymentResponse {
        case noTimeAdded(_ expiry: Date)
        case timeAdded(_ timeAdded: Int, _ newExpiry: Date)

        var newExpiry: Date {
            switch self {
            case .noTimeAdded(let expiry), .timeAdded(_, let expiry):
                return expiry
            }
        }

        var timeAdded: TimeInterval {
            switch self {
            case .noTimeAdded:
                return 0
            case .timeAdded(let timeAdded, _):
                return TimeInterval(timeAdded)
            }
        }

        /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
        var formattedTimeAdded: String? {
            let formatter = DateComponentsFormatter()
            formatter.allowedUnits = [.day, .hour]
            formatter.unitsStyle = .full

            return formatter.string(from: self.timeAdded)
        }
    }

    fileprivate struct CreateApplePaymentRawResponse: Decodable {
        let timeAdded: Int
        let newExpiry: Date
    }

    struct ProblemReportRequest: Encodable {
        let address: String
        let message: String
        let log: String
        let metadata: [String: String]
    }

}
