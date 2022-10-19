//
//  RESTRequestHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol RESTRequestHandler {
    func createURLRequest(
        endpoint: AnyIPEndpoint,
        authorization: REST.Authorization?
    ) throws -> REST.Request

    var authorizationProvider: RESTAuthorizationProvider? { get }
}

extension REST {
    public struct Request {
        var urlRequest: URLRequest
        var pathTemplate: URLPathTemplate
    }

    internal final class AnyRequestHandler: RESTRequestHandler {
        private let _createURLRequest: (AnyIPEndpoint, REST.Authorization?) throws -> REST.Request

        internal let authorizationProvider: RESTAuthorizationProvider?

        internal init(createURLRequest: @escaping (AnyIPEndpoint) throws -> REST.Request) {
            _createURLRequest = { endpoint, authorization in
                return try createURLRequest(endpoint)
            }
            authorizationProvider = nil
        }

        internal init(
            createURLRequest: @escaping (AnyIPEndpoint, REST.Authorization) throws -> REST.Request,
            authorizationProvider: RESTAuthorizationProvider
        ) {
            _createURLRequest = { endpoint, authorization in
                return try createURLRequest(endpoint, authorization!)
            }
            self.authorizationProvider = authorizationProvider
        }

        internal func createURLRequest(
            endpoint: AnyIPEndpoint,
            authorization: REST.Authorization?
        ) throws -> REST.Request {
            return try _createURLRequest(endpoint, authorization)
        }
    }
}
