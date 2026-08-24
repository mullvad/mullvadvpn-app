// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

class TestRouterAPIClient {
    static let baseURL = URL(string: Bundle(for: FirewallClient.self).infoDictionary?["FirewallApiBaseURL"] as! String)!

    /// Gets the IP address of the device under test
    public func getDeviceIPAddress() throws -> String {
        let deviceIPURL = TestRouterAPIClient.baseURL.appendingPathComponent("own-ip")
        let request = URLRequest(url: deviceIPURL)
        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )
        nonisolated(unsafe) var deviceIPAddress = ""
        nonisolated(unsafe) var requestError: Error?

        let dataTask = URLSession.shared.dataTask(with: request) { data, _, _ in
            defer { completionHandlerInvokedExpectation.fulfill() }
            guard let data else {
                requestError = NetworkingError.internalError(reason: "Could not get device IP")
                return
            }

            deviceIPAddress = String(data: data, encoding: .utf8)!
        }

        dataTask.resume()

        let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 5)
        if waitResult != .completed {
            XCTFail("Failed to get device IP address - timeout")
        }

        if let requestError {
            throw requestError
        }

        return deviceIPAddress
    }
}
