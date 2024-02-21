//
//  RESTAccountsProxy.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol RESTAccountHandling {
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<REST.NewAccountData>
    ) -> Cancellable

    func getAccountData(accountNumber: String) -> any RESTRequestExecutor<Account>

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable
}

extension REST {
    public final class AccountsProxy: Proxy<AuthProxyConfiguration>, RESTAccountHandling {
        public init(configuration: AuthProxyConfiguration) {
            super.init(
                name: "AccountsProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/accounts/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        public func createAccount(
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<NewAccountData>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                try self.requestFactory.createRequest(
                    endpoint: endpoint,
                    method: .post,
                    pathTemplate: "accounts"
                )
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: NewAccountData.self,
                with: responseDecoder
            )

            let executor = makeRequestExecutor(
                name: "create-account",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completion)
        }

        public func getAccountData(accountNumber: String) -> any RESTRequestExecutor<Account> {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var requestBuilder = try self.requestFactory.createRequestBuilder(
                        endpoint: endpoint,
                        method: .get,
                        pathTemplate: "accounts/me"
                    )

                    requestBuilder.setAuthorization(authorization)

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler = REST.defaultResponseHandler(
                decoding: Account.self,
                with: responseDecoder
            )

            return makeRequestExecutor(
                name: "get-my-account",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )
        }

        public func deleteAccount(
            accountNumber: String,
            retryStrategy: RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Void>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(createURLRequest: { endpoint, authorization in
                var requestBuilder = try self.requestFactory.createRequestBuilder(
                    endpoint: endpoint,
                    method: .delete,
                    pathTemplate: "accounts/me"
                )
                requestBuilder.setAuthorization(authorization)
                requestBuilder.addValue(accountNumber, forHTTPHeaderField: "Mullvad-Account-Number")

                return requestBuilder.getRequest()
            }, authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber))

            let responseHandler = AnyResponseHandler { response, data -> ResponseHandlerResult<Void> in
                let statusCode = HTTPStatus(rawValue: response.statusCode)

                switch statusCode {
                case let statusCode where statusCode.isSuccess:
                    return .success(())
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
                name: "delete-my-account",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completion)
        }
    }

    public struct NewAccountData: Decodable {
        public let id: String
        public let expiry: Date
        public let maxPorts: Int
        public let canAddPorts: Bool
        public let maxDevices: Int
        public let canAddDevices: Bool
        public let number: String
    }
}

extension REST.NewAccountData {
    public static func mockValue() -> REST.NewAccountData {
        return REST.NewAccountData(
            id: UUID().uuidString,
            expiry: Date().addingTimeInterval(3600),
            maxPorts: 2,
            canAddPorts: false,
            maxDevices: 5,
            canAddDevices: false,
            number: "1234567890123456"
        )
    }
}
