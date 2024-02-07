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

    /// Get IP address of the iOS device under test
    static func getIPAddress() -> String? {
        var address: String?
        // Get list of all interfaces on the local machine:
        var ifaddr: UnsafeMutablePointer<ifaddrs>?
        guard getifaddrs(&ifaddr) == 0 else { return nil }
        guard let firstAddr = ifaddr else { return nil }

        // For each interface
        for ptr in sequence(first: firstAddr, next: { $0.pointee.ifa_next }) {
            let flags = Int32(ptr.pointee.ifa_flags)
            let addr = ptr.pointee.ifa_addr.pointee

            // Check for running IPv4, IPv6 interfaces. Skip the loopback interface.
            if (
                flags & (
                    IFF_UP |
                        IFF_RUNNING |
                        IFF_LOOPBACK
                )
            ) == (
                IFF_UP |
                    IFF_RUNNING
            ),
                addr.sa_family == UInt8(AF_INET) || addr.sa_family == UInt8(AF_INET6) {
                // Check if interface is en0 which is the WiFi connection on the iPhone
                let name = String(cString: ptr.pointee.ifa_name)
                if name == "en0" {
                    // Convert interface address to a human readable string:
                    var hostname = [CChar](repeating: 0, count: Int(NI_MAXHOST))
                    getnameinfo(
                        ptr.pointee.ifa_addr,
                        socklen_t(addr.sa_len),
                        &hostname,
                        socklen_t(hostname.count),
                        nil,
                        socklen_t(0),
                        NI_NUMERICHOST
                    )
                    address = String(cString: hostname)
                }
            }
        }

        freeifaddrs(ifaddr)

        return address
    }

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
                fatalError("Failed to create firewall rule - timeout")
            } else {
                if let response = requestResponse as? HTTPURLResponse {
                    if response.statusCode != 201 {
                        fatalError("Failed to create firewall rule - unexpected server response")
                    }
                }

                if let error = requestError {
                    fatalError("Failed to create firewall rule - encountered error \(error.localizedDescription)")
                }
            }
        } catch {
            fatalError("Failed to create firewall rule - couldn't serialize JSON")
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
            fatalError("Failed to remove firewall rules - timeout")
        } else {
            if let response = requestResponse as? HTTPURLResponse, response.statusCode != 200 {
                fatalError("Failed to remove firewall rules - unexpected server response")
            }

            if let error = requestError {
                fatalError("Failed to remove firewall rules - encountered error \(error.localizedDescription)")
            }
        }
    }
}
