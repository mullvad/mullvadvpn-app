//
//  RESTTransportStub.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2024-01-22.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadTypes
import XCTest

struct RESTTransportStub: RESTTransport {
    let name = "transport-stub"

    var data: Data?
    var response: URLResponse?
    var error: Error?

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) -> Cancellable {
        completion(data, response, error)
        return AnyCancellable()
    }
}
