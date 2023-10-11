//
//  URLRequestProxyStub.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore
import XCTest

struct URLRequestProxyStub: URLRequestProxyProtocol {
    var sendRequestExpectation: XCTestExpectation?
    var cancelRequestExpectation: XCTestExpectation?

    func sendRequest(
        _ proxyRequest: PacketTunnelCore.ProxyURLRequest,
        completionHandler: @escaping @Sendable (PacketTunnelCore.ProxyURLResponse) -> Void
    ) {
        sendRequestExpectation?.fulfill()
    }

    func sendRequest(_ proxyRequest: PacketTunnelCore.ProxyURLRequest) async -> PacketTunnelCore.ProxyURLResponse {
        sendRequestExpectation?.fulfill()
        return ProxyURLResponse(data: nil, response: nil, error: nil)
    }

    func cancelRequest(identifier: UUID) {
        cancelRequestExpectation?.fulfill()
    }
}
