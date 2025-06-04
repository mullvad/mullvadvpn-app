//
//  TestRouterAPIClient.swift
//  MullvadVPN
//
//  Created by Niklas Berglund on 2024-12-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

class TestRouterAPIClient {
    // swiftlint:disable:next force_cast
    static let baseURL = URL(string: Bundle(for: FirewallClient.self).infoDictionary?["FirewallApiBaseURL"] as! String)!

    /// Gets the IP address of the device under test
    public func getDeviceIPAddress() throws -> String {
        let deviceIPURL = TestRouterAPIClient.baseURL.appendingPathComponent("own-ip")
        let request = URLRequest(url: deviceIPURL)
        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )
        var deviceIPAddress = ""
        var requestError: Error?

        let dataTask = URLSession.shared.dataTask(with: request) { data, _, _ in
            defer { completionHandlerInvokedExpectation.fulfill() }
            guard let data else {
                requestError = NetworkingError.internalError(reason: "Could not get device IP")
                return
            }

            deviceIPAddress = String(data: data, encoding: .utf8)!
        }

        dataTask.resume()

        let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)
        if waitResult != .completed {
            XCTFail("Failed to get device IP address - timeout")
        }

        if let requestError {
            throw requestError
        }

        return deviceIPAddress
    }
}
