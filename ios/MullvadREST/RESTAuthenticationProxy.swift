//
//  RESTAuthenticationProxy.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    public final class AuthenticationProxy: Proxy<ProxyConfiguration> {
        public init(configuration: ProxyConfiguration) {
            super.init(
                name: "AuthenticationProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/auth/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        func getAccessToken(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<AccessTokenData>
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

            return addOperation(
                name: "get-access-token",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }
    }

    struct AccessTokenData: Decodable {
        let accessToken: String
        let expiry: Date
    }

    private struct AccessTokenRequest: Encodable {
        let accountNumber: String
    }
}
