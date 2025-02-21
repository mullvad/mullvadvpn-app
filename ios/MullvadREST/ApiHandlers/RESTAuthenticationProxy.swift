//
//  RESTAuthenticationProxy.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    public final class AuthenticationProxy: Proxy<ProxyConfiguration>, @unchecked Sendable {
        public init(configuration: ProxyConfiguration) {
            super.init(
                name: "AuthenticationProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/auth/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: Coding.makeJSONDecoder(),
                requestEncoder: Coding.makeJSONEncoder()
            )
        }

        public func getAccessToken(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<AccessTokenData>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler { endpoint in
                var requestBuilder = try self.requestFactory.createRequestBuilder(
                    endpoint: endpoint,
                    method: .post,
                    pathTemplate: "token"
                )

                let request = AccessTokenRequest(accountNumber: accountNumber)

                try requestBuilder.setHTTPBody(value: request)

                return requestBuilder.getRequest()
            }

            let responseHandler = REST.defaultResponseHandler(
                decoding: AccessTokenData.self,
                with: responseDecoder
            )

            let executor = makeRequestExecutor(
                name: "get-access-token",
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completion)
        }
    }

    public struct AccessTokenData: Decodable, Sendable {
        let accessToken: String
        let expiry: Date
    }

    private struct AccessTokenRequest: Encodable, Sendable {
        let accountNumber: String
    }
}
