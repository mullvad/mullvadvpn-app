//
//  RESTAccountsProxy.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    public final class AccountsProxy: Proxy<AuthProxyConfiguration> {
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
            completion: @escaping CompletionHandler<NewAccountData>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                return try self.requestFactory.createRequest(
                    endpoint: endpoint,
                    method: .post,
                    pathTemplate: "accounts"
                )
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: NewAccountData.self,
                with: responseDecoder
            )

            return addOperation(
                name: "create-account",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        public func getAccountData(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<AccountData>
        ) -> Cancellable {
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
                decoding: AccountData.self,
                with: responseDecoder
            )

            return addOperation(
                name: "get-my-account",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }
    }

    public struct AccountData: Decodable {
        public let id: String
        public let expiry: Date
        public let maxPorts: Int
        public let canAddPorts: Bool
        public let maxDevices: Int
        public let canAddDevices: Bool
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
