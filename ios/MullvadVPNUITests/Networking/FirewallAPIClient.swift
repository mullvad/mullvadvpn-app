//
//  FirewallClient.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SystemConfiguration
import UIKit
import XCTest

class FirewallAPIClient {
    // swiftlint:disable force_cast
    let baseURL = URL(
        string:
        Bundle(for: FirewallAPIClient.self).infoDictionary?["FirewallApiBaseURL"] as! String
    )!
    let testDeviceIdentifier = Bundle(for: FirewallAPIClient.self).infoDictionary?["TestDeviceIdentifier"] as! String
    // swiftlint:enable force_cast

    lazy var sessionIdentifier = "urn:uuid:" + testDeviceIdentifier

    /// Create a new rule associated to the device under test
    public func createRule(_ firewallRule: FirewallRule) {
        let createRuleURL = baseURL.appendingPathComponent("rule")

        var request = URLRequest(url: createRuleURL)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let dataDictionary: [String: Any] = [
            "label": sessionIdentifier,
            "from": firewallRule.fromIPAddress, // Deprecated, replaced by "src"
            "to": firewallRule.toIPAddress, // Deprectated, replaced by "dst"
            "src": firewallRule.fromIPAddress,
            "dst": firewallRule.toIPAddress,
            "protocols": firewallRule.protocolsAsStringArray(),
        ]

        var requestError: Error?
        var requestResponse: URLResponse?
        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: dataDictionary)
            request.httpBody = jsonData

            let dataTask = URLSession.shared.dataTask(with: request) { _, response, error in
                requestError = error
                requestResponse = response
                completionHandlerInvokedExpectation.fulfill()
            }

            dataTask.resume()

            let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)

            if waitResult != .completed {
                XCTFail("Failed to create firewall rule - timeout")
            } else {
                if let response = requestResponse as? HTTPURLResponse {
                    if response.statusCode != 201 {
                        XCTFail("Failed to create firewall rule - unexpected server response")
                    }
                }

                if let error = requestError {
                    XCTFail("Failed to create firewall rule - encountered error \(error.localizedDescription)")
                }
            }
        } catch {
            XCTFail("Failed to create firewall rule - couldn't serialize JSON")
        }
    }

    /// Gets the IP address of the device under test
    public func getDeviceIPAddress() throws -> String {
        let deviceIPURL = baseURL.appendingPathComponent("own-ip")
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

    /// Remove all firewall rules associated to this device under test
    public func removeRules() {
        let removeRulesURL = baseURL.appendingPathComponent("remove-rules/\(sessionIdentifier)")

        var request = URLRequest(url: removeRulesURL)
        request.httpMethod = "DELETE"

        var requestResponse: URLResponse?
        var requestError: Error?
        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )

        let dataTask = URLSession.shared.dataTask(with: request) { _, response, error in
            requestResponse = response
            requestError = error
            completionHandlerInvokedExpectation.fulfill()
        }

        dataTask.resume()

        let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)

        if waitResult != .completed {
            XCTFail("Failed to remove firewall rules - timeout")
        } else {
            if let response = requestResponse as? HTTPURLResponse, response.statusCode != 200 {
                XCTFail("Failed to remove firewall rules - unexpected server response")
            }

            if let error = requestError {
                XCTFail("Failed to remove firewall rules - encountered error \(error.localizedDescription)")
            }
        }
    }
}
