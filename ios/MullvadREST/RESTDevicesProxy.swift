//
//  RESTDevicesProxy.swift
//  MullvadREST
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PublicKey

extension REST {
    public final class DevicesProxy: Proxy<AuthProxyConfiguration> {
        public init(configuration: AuthProxyConfiguration) {
            super.init(
                name: "DevicesProxy",
                configuration: configuration,
                requestFactory: RequestFactory.withDefaultAPICredentials(
                    pathPrefix: "/accounts/v1",
                    bodyEncoder: Coding.makeJSONEncoder()
                ),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        /// Fetch device by identifier.
        /// The completion handler receives `nil` if device is not found.
        public func getDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<Device>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var path: URLPathTemplate = "devices/{id}"

                    try path.addPercentEncodedReplacement(
                        name: "id",
                        value: identifier,
                        allowedCharacters: .urlPathAllowed
                    )

                    var requestBuilder = try self.requestFactory.createRequestBuilder(
                        endpoint: endpoint,
                        method: .get,
                        pathTemplate: path
                    )

                    requestBuilder.setAuthorization(authorization)

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler =
                AnyResponseHandler { response, data -> ResponseHandlerResult<Device> in
                    let httpStatus = HTTPStatus(rawValue: response.statusCode)

                    if httpStatus.isSuccess {
                        return .decoding {
                            return try self.responseDecoder.decode(Device.self, from: data)
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
                name: "get-device",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        /// Fetch a list of created devices.
        public func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<[Device]>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var requestBuilder = try self.requestFactory.createRequestBuilder(
                        endpoint: endpoint,
                        method: .get,
                        pathTemplate: "devices"
                    )

                    requestBuilder.setAuthorization(authorization)

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
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

        /// Create new device.
        /// The completion handler will receive a `CreateDeviceResponse.created(Device)` on success.
        /// Other `CreateDeviceResponse` variants describe errors.
        public func createDevice(
            accountNumber: String,
            request: CreateDeviceRequest,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<Device>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var requestBuilder = try self.requestFactory.createRequestBuilder(
                        endpoint: endpoint,
                        method: .post,
                        pathTemplate: "devices"
                    )
                    requestBuilder.setAuthorization(authorization)

                    try requestBuilder.setHTTPBody(value: request)

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler = REST.defaultResponseHandler(
                decoding: Device.self,
                with: responseDecoder
            )

            return addOperation(
                name: "create-device",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        /// Delete device by identifier.
        /// The completion handler will receive `true` if device is successfully removed,
        /// otherwise `false` if device is not found or already removed.
        public func deleteDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<Bool>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var path: URLPathTemplate = "devices/{id}"

                    try path.addPercentEncodedReplacement(
                        name: "id",
                        value: identifier,
                        allowedCharacters: .urlPathAllowed
                    )

                    var requestBuilder = try self.requestFactory
                        .createRequestBuilder(
                            endpoint: endpoint,
                            method: .delete,
                            pathTemplate: path
                        )

                    requestBuilder.setAuthorization(authorization)

                    return requestBuilder.getRequest()
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler =
                AnyResponseHandler { response, data -> ResponseHandlerResult<Bool> in
                    let statusCode = HTTPStatus(rawValue: response.statusCode)

                    switch statusCode {
                    case let statusCode where statusCode.isSuccess:
                        return .success(true)

                    case .notFound:
                        return .success(false)

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
                name: "delete-device",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        /// Rotate device key
        public func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: PublicKey,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping CompletionHandler<Device>
        ) -> Cancellable {
            let requestHandler = AnyRequestHandler(
                createURLRequest: { endpoint, authorization in
                    var path: URLPathTemplate = "devices/{id}/pubkey"

                    try path.addPercentEncodedReplacement(
                        name: "id",
                        value: identifier,
                        allowedCharacters: .urlPathAllowed
                    )

                    var requestBuilder = try self.requestFactory
                        .createRequestBuilder(
                            endpoint: endpoint,
                            method: .put,
                            pathTemplate: path
                        )

                    requestBuilder.setAuthorization(authorization)

                    let request = RotateDeviceKeyRequest(
                        publicKey: publicKey
                    )
                    try requestBuilder.setHTTPBody(value: request)

                    let urlRequest = requestBuilder.getRequest()

                    return urlRequest
                },
                authorizationProvider: createAuthorizationProvider(accountNumber: accountNumber)
            )

            let responseHandler = REST.defaultResponseHandler(
                decoding: Device.self,
                with: responseDecoder
            )

            return addOperation(
                name: "rotate-device-key",
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }
    }

    public struct CreateDeviceRequest: Encodable {
        let publicKey: PublicKey
        let hijackDNS: Bool

        public init(publicKey: PublicKey, hijackDNS: Bool) {
            self.publicKey = publicKey
            self.hijackDNS = hijackDNS
        }

        private enum CodingKeys: String, CodingKey {
            case hijackDNS = "hijackDns"
            case publicKey = "pubkey"
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)

            try container.encode(publicKey.base64Key, forKey: .publicKey)
            try container.encode(hijackDNS, forKey: .hijackDNS)
        }
    }

    private struct RotateDeviceKeyRequest: Encodable {
        let publicKey: PublicKey

        private enum CodingKeys: String, CodingKey {
            case publicKey = "pubkey"
        }

        func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)

            try container.encode(publicKey.base64Key, forKey: .publicKey)
        }
    }

    public struct Device: Decodable {
        public let id: String
        public let name: String
        public let pubkey: PublicKey
        public let hijackDNS: Bool
        public let created: Date
        public let ipv4Address: IPAddressRange
        public let ipv6Address: IPAddressRange
        public let ports: [Port]

        private enum CodingKeys: String, CodingKey {
            case hijackDNS = "hijackDns"
            case id, name, pubkey, created, ipv4Address, ipv6Address, ports
        }
    }

    public struct Port: Decodable {
        public let id: String
    }
}
