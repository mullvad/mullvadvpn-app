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
            "from": firewallRule.fromIPAddress,
            "to": firewallRule.toIPAddress,
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

    /// Remove all firewall rules associated to this device under test
    public func removeRules() {
        let removeRulesURL = baseURL.appendingPathComponent("remove-rules/\(sessionIdentifier)")

        var request = URLRequest(url: removeRulesURL)
        request.httpMethod = "DELETE"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

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
