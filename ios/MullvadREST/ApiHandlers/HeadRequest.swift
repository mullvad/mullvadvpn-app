//
//  HeadRequest.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2024-01-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    public struct HeadRequest {
        let transport: RESTTransport

        public init(transport: RESTTransport) {
            self.transport = transport
        }

        /// Executes an HTTP `HEAD` request to the "api-addrs" endpoint.
        ///
        /// - Parameter completion: Completes with `nil` if the request was successful, and `Error` otherwise.
        /// - Returns: A cancellable token to cancel the request inflight.
        public func makeRequest(completion: @escaping (Swift.Error?) -> Void) -> Cancellable {
            do {
                let factory = RequestFactory(
                    hostname: defaultAPIHostname,
                    pathPrefix: "/app/v1",
                    networkTimeout: defaultAPINetworkTimeout,
                    bodyEncoder: JSONEncoder()
                )
                var request = try factory.createRequest(
                    endpoint: defaultAPIEndpoint,
                    method: .head,
                    pathTemplate: "api-addrs"
                )
                request.urlRequest.cachePolicy = .reloadIgnoringLocalCacheData

                return transport.sendRequest(request.urlRequest) { _, response, error in
                    // Any response in the form of `HTTPURLResponse` means that the API was reached successfully
                    // and implying an HTTP server is running, therefore the test is considered successful.
                    guard response is HTTPURLResponse else {
                        completion(error)
                        return
                    }
                    completion(nil)
                }

            } catch {
                completion(error)
            }
            return AnyCancellable()
        }
    }
}
