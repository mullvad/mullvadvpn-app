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
import XCTest

@testable import MullvadREST
@testable import MullvadTypes

final class APIRequestProxyTests: XCTestCase {
    /// `APIRequestProxy` guards its request bookkeeping with `dispatchPrecondition`, which traps when the request is
    /// registered off the proxy queue after awaiting the transport.
    func testSendRequestRegistersRequestOnProxyQueue() async {
        let transportProvider = await REST.AnyAPITransportProvider { APITransportStub() }
        let proxy = APIRequestProxy(
            dispatchQueue: DispatchQueue(label: "APIRequestProxyTests"),
            transportProvider: transportProvider
        )

        let response = await proxy.sendRequest(ProxyAPIRequest(id: UUID(), request: .getAddressList(.noRetry)))

        XCTAssertNil(response.error)
    }
}

private struct APITransportStub: APITransportProtocol {
    let name = "stub-transport"

    func sendRequest(
        _ request: APIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) throws -> Cancellable {
        completion(ProxyAPIResponse(data: nil, error: nil))
        return AnyCancellable()
    }

    func sendRequest(_ request: APIRequest) async throws -> ProxyAPIResponse {
        ProxyAPIResponse(data: nil, error: nil)
    }
}
