//
//  RESTTransportStub.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2024-01-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadREST
@testable import MullvadTypes

struct RESTTransportStub: RESTTransport {
    let name = "transport-stub"

    var data: Data?
    var response: URLResponse?
    var error: Error?

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping @Sendable (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        completion(data, response, error)
        return AnyCancellable()
    }
}
