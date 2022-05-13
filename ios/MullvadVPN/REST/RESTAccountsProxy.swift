//
//  RESTAccountsProxy.swift
//  MullvadVPN
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    class AccountsProxy: Proxy<AuthProxyConfiguration> {
        init(configuration: AuthProxyConfiguration) {
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

        func createAccount(
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

        func getAccountData(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<AccountData>
        ) -> Cancellable
        {
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
                requestAuthorization: { completion in
                    return self.configuration.accessTokenManager
                        .getAccessToken(
                            accountNumber: accountNumber,
                            retryStrategy: retryStrategy
                        ) { operationCompletion in
                            completion(operationCompletion.map { tokenData in
                                return .accessToken(tokenData.accessToken)
                            })
                        }
                }
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

    struct AccountData: Decodable {
        let id: String
        let expiry: Date
        let maxPorts: Int
        let canAddPorts: Bool
        let maxDevices: Int
        let canAddDevices: Bool
    }

    struct NewAccountData: Decodable {
        let id: String
        let expiry: Date
        let maxPorts: Int
        let canAddPorts: Bool
        let maxDevices: Int
        let canAddDevices: Bool
        let number: String
    }
}
