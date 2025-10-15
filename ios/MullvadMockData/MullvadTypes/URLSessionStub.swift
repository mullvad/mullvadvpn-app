//
//  URLSessionStub.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2023-10-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

class URLSessionStub: URLSessionProtocol {
    var response: (Data, URLResponse)

    init(response: (Data, URLResponse)) {
        self.response = response
    }

    func data(for request: URLRequest) async throws -> (Data, URLResponse) {
        return response
    }
}
