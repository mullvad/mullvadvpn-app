// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
