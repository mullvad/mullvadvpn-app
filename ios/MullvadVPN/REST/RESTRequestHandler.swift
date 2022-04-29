//
//  RESTRequestHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol RESTRequestHandler {
    typealias AuthorizationCompletion = (OperationCompletion<REST.Authorization, REST.Error>) -> Void

    func createURLRequest(endpoint: AnyIPEndpoint, authorization: REST.Authorization?) -> Result<URLRequest, REST.Error>
    func requestAuthorization(completion: @escaping AuthorizationCompletion) -> REST.AuthorizationResult
}

extension REST {

    enum AuthorizationResult {
        /// There is no requirement for authorizing this request.
        case noRequirement

        /// Authorization request is initiated.
        /// Associated value contains a handle that can be used to cancel
        /// the request.
        case pending(Cancellable)
    }

    final class AnyRequestHandler: RESTRequestHandler {
        private let _createURLRequest: (AnyIPEndpoint, REST.Authorization?) -> Result<URLRequest, REST.Error>
        private let _requestAuthorization: ((@escaping AuthorizationCompletion) -> AuthorizationResult)?

        init(createURLRequest: @escaping (AnyIPEndpoint) -> Result<URLRequest, REST.Error>) {
            _createURLRequest = { endpoint, authorization in
                createURLRequest(endpoint)
            }
            _requestAuthorization = nil
        }

        init(
            createURLRequest: @escaping (AnyIPEndpoint, REST.Authorization) -> Result<URLRequest, REST.Error>,
            requestAuthorization: @escaping (@escaping AuthorizationCompletion) -> Cancellable
        ) {
            _createURLRequest = { endpoint, authorization in
                return createURLRequest(endpoint, authorization!)
            }
            _requestAuthorization = { completion in
                return .pending(requestAuthorization(completion))
            }
        }

        func createURLRequest(
            endpoint: AnyIPEndpoint,
            authorization: REST.Authorization?
        ) -> Result<URLRequest, REST.Error> {
            return _createURLRequest(endpoint, authorization)
        }

        func requestAuthorization(
            completion: @escaping (OperationCompletion<REST.Authorization, REST.Error>) -> Void
        ) -> REST.AuthorizationResult {
            return _requestAuthorization?(completion) ?? .noRequirement
        }
    }

}
