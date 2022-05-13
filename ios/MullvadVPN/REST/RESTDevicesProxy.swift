//
//  RESTDevicesProxy.swift
//  MullvadVPN
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKitTypes.PublicKey
import struct WireGuardKitTypes.IPAddressRange

extension REST {
    class DevicesProxy: Proxy<AuthProxyConfiguration> {
        init(configuration: AuthProxyConfiguration) {
            super.init(
                name: "DevicesProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/accounts/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: ResponseDecoder(
                    decoder: Coding.makeJSONDecoder()
                )
            )
        }

        func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<[Device]>
        ) -> Cancellable
        {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var requestBuilder = self.requestFactory.createURLRequestBuilder(
                        endpoint: endpoint,
                        method: .get,
                        path: "/devices"
                    )

                    requestBuilder.setAuthorization(authorization)

                    return .success(requestBuilder.getURLRequest())
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
                decoding: [Device].self,
                with: responseDecoder
            )

            return addOperation(
                name: "get-devices",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

    }

    struct Device: Decodable {
        let id: String
        let name: String
        let pubkey: Data
        let hijackDNS: Bool
        let created: Date
        let ipv4Address: IPAddressRange
        let ipv6Address: IPAddressRange
        let ports: [Port]

        private enum CodingKeys: String, CodingKey {
            case hijackDNS = "hijackDns"
            case id, name, pubkey, created, ipv4Address, ipv6Address, ports
        }
    }

    struct Port: Decodable {
        let id: String
    }

}
