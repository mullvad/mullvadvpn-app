//
//  APIRequestProxyStub.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import PacketTunnelCore
import XCTest

struct APIRequestProxyStub: APIRequestProxyProtocol {
    var sendRequestExpectation: XCTestExpectation?
    var cancelRequestExpectation: XCTestExpectation?

    func sendRequest(
        _ proxyRequest: ProxyAPIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) {
        sendRequestExpectation?.fulfill()
    }

    func sendRequest(_ proxyRequest: ProxyAPIRequest) async -> ProxyAPIResponse {
        sendRequestExpectation?.fulfill()
        return ProxyAPIResponse(data: nil, error: nil)
    }

    func cancelRequest(identifier: UUID) {
        cancelRequestExpectation?.fulfill()
    }
}
