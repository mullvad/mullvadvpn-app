//
//  Networking.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import XCTest

enum NetworkTransportProtocol: String, Codable {
    case TCP = "tcp"
    case UDP = "udp"
    case ICMP = "icmp"
}

enum NetworkingError: Error {
    case notConfiguredError
    case internalError(reason: String)
}

struct DNSServerEntry: Decodable {
    let organization: String
    let mullvad_dns: Bool
}

/// Class with methods for verifying network connectivity
class Networking {
    /// Get configured ad serving domain
    private static func getAdServingDomain() throws -> String {
        guard let adServingDomain = Bundle(for: Networking.self)
            .infoDictionary?["AdServingDomain"] as? String else {
            throw NetworkingError.notConfiguredError
        }

        return adServingDomain
    }

    /// Check whether host and port is reachable by attempting to connect a socket
    private static func canConnectSocket(host: String, port: String) throws -> Bool {
        let socketHost = NWEndpoint.Host(host)
        let socketPort = try XCTUnwrap(NWEndpoint.Port(port))
        let connection = NWConnection(host: socketHost, port: socketPort, using: .tcp)
        var connectionError: Error?

        let connectionStateDeterminedExpectation = XCTestExpectation(
            description: "Completion handler for the reach ad serving domain request is invoked"
        )

        connection.stateUpdateHandler = { state in
            print("State: \(state)")

            switch state {
            case let .failed(error):
                connection.cancel()
                connectionError = error
                connectionStateDeterminedExpectation.fulfill()
            case .ready:
                connection.cancel()
                connectionStateDeterminedExpectation.fulfill()
            default:
                break
            }
        }

        connection.start(queue: .global())
        let waitResult = XCTWaiter.wait(for: [connectionStateDeterminedExpectation], timeout: 15)

        if waitResult != .completed || connectionError != nil {
            return false
        }

        return true
    }

    /// Get configured domain to use for Internet connectivity checks
    public static func getAlwaysReachableDomain() throws -> String {
        guard let shouldBeReachableDomain = Bundle(for: Networking.self)
            .infoDictionary?["ShouldBeReachableDomain"] as? String else {
            throw NetworkingError.notConfiguredError
        }

        return shouldBeReachableDomain
    }

    public static func getAlwaysReachableIPAddress() -> String {
        guard let shouldBeReachableIPAddress = Bundle(for: Networking.self)
            .infoDictionary?["ShouldBeReachableIPAddress"] as? String else {
            XCTFail("Should be reachable IP address not configured")
            return String()
        }

        return shouldBeReachableIPAddress
    }

    /// Verify API can be accessed by attempting to connect a socket to the configured API host and port
    public static func verifyCanAccessAPI() throws {
        let apiIPAddress = try MullvadAPIWrapper.getAPIIPAddress()
        let apiPort = try MullvadAPIWrapper.getAPIPort()
        XCTAssertTrue(
            try canConnectSocket(host: apiIPAddress, port: apiPort),
            "Failed to verify that API can be accessed"
        )
    }

    /// Verify API cannot be accessed by attempting to connect a socket to the configured API host and port
    public static func verifyCannotAccessAPI() throws {
        let apiIPAddress = try MullvadAPIWrapper.getAPIIPAddress()
        let apiPort = try MullvadAPIWrapper.getAPIPort()
        XCTAssertFalse(
            try canConnectSocket(host: apiIPAddress, port: apiPort),
            "Failed to verify that API cannot be accessed"
        )
    }

    /// Verify that the device has Internet connectivity
    public static func verifyCanAccessInternet() throws {
        XCTAssertTrue(
            try canConnectSocket(host: getAlwaysReachableDomain(), port: "80"),
            "Failed to verify that the Internet can be acccessed"
        )
    }

    /// Verify that the device does not have Internet connectivity
    public static func verifyCannotAccessInternet() throws {
        XCTAssertFalse(
            try canConnectSocket(host: getAlwaysReachableDomain(), port: "80"),
            "Failed to verify that the Internet cannot be accessed"
        )
    }

    /// Verify that an ad serving domain is reachable by making sure a connection can be established on port 80
    public static func verifyCanReachAdServingDomain() throws {
        XCTAssertTrue(
            try Self.canConnectSocket(host: try Self.getAdServingDomain(), port: "80"),
            "Failed to verify that ad serving domain can be accessed"
        )
    }

    /// Verify that an ad serving domain is NOT reachable by making sure a connection can not be established on port 80
    public static func verifyCannotReachAdServingDomain() throws {
        XCTAssertFalse(
            try Self.canConnectSocket(host: try Self.getAdServingDomain(), port: "80"),
            "Failed to verify that ad serving domain cannot be accessed"
        )
    }

    /// Verify that the expected DNS server is used by verifying provider name and whether it is a Mullvad DNS server or not
    public static func verifyDNSServerProvider(_ providerName: String, isMullvad: Bool) throws {
        guard let mullvadDNSLeakURL = URL(string: "https://am.i.mullvad.net/dnsleak") else {
            throw NetworkingError.internalError(reason: "Failed to create URL object")
        }

        var request = URLRequest(url: mullvadDNSLeakURL)
        request.setValue("application/json", forHTTPHeaderField: "accept")

        var requestData: Data?
        var requestResponse: URLResponse?
        var requestError: Error?
        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )

        do {
            let dataTask = URLSession.shared.dataTask(with: request) { data, response, error in
                requestData = data
                requestResponse = response
                requestError = error
                completionHandlerInvokedExpectation.fulfill()
            }

            dataTask.resume()

            let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)

            if waitResult != .completed {
                XCTFail("Failed to verify DNS server provider - timeout")
            } else {
                if let response = requestResponse as? HTTPURLResponse {
                    if response.statusCode != 200 {
                        XCTFail("Failed to verify DNS server provider - unexpected server response")
                    }
                }

                if let error = requestError {
                    XCTFail("Failed to verify DNS server provider - encountered error \(error.localizedDescription)")
                }

                if let requestData = requestData {
                    let dnsServerEntries = try JSONDecoder().decode([DNSServerEntry].self, from: requestData)
                    XCTAssertGreaterThanOrEqual(dnsServerEntries.count, 1)

                    for dnsServerEntry in dnsServerEntries {
                        XCTAssertEqual(dnsServerEntry.organization, providerName, "Expected organization name")
                        XCTAssertEqual(
                            dnsServerEntry.mullvad_dns,
                            isMullvad,
                            "Verifying that it is or isn't a Mullvad DNS server"
                        )
                    }
                }
            }
        } catch {
            XCTFail("Failed to verify DNS server provider - couldn't serialize JSON")
        }
    }

    public static func verifyConnectedThroughMullvad() throws {
        let mullvadConnectionJsonEndpoint = try XCTUnwrap(
            Bundle(for: Networking.self)
                .infoDictionary?["AmIJSONUrl"] as? String,
            "Read am I JSON URL from Info"
        )
        guard let url = URL(string: mullvadConnectionJsonEndpoint) else {
            XCTFail("Failed to unwrap URL")
            return
        }

        let request = URLRequest(url: url)
        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )

        let dataTask = URLSession.shared.dataTask(with: request) { data, response, error in
            if let response = response as? HTTPURLResponse {
                if response.statusCode != 200 {
                    XCTFail("Request to connection check API failed - unexpected server response")
                }
            }

            if let error = error {
                XCTFail("Request to connection check API failed - encountered error \(error.localizedDescription)")
            }

            guard let data = data else {
                XCTFail("Didn't receive any data")
                return
            }

            do {
                let jsonObject = try JSONSerialization.jsonObject(with: data)

                if let dictionary = jsonObject as? [String: Any] {
                    guard let isConnectedThroughMullvad = dictionary["mullvad_exit_ip"] as? Bool else {
                        XCTFail("Unexpected JSON format")
                        return
                    }

                    XCTAssertTrue(isConnectedThroughMullvad)
                }
            } catch {
                XCTFail("Failed to verify whether connected through Mullvad or not")
            }

            completionHandlerInvokedExpectation.fulfill()
        }

        dataTask.resume()

        let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)

        if waitResult != .completed {
            XCTFail("Request to connection check API failed - timeout")
        }
    }

    public static func verifyCanAccessLocalNetwork() throws {
        let apiIPAddress = "192.168.105.1"
        let apiPort = "80"
        XCTAssertTrue(
            try canConnectSocket(host: apiIPAddress, port: apiPort),
            "Failed to verify that local network can be accessed"
        )
    }

    public static func verifyCannotAccessLocalNetwork() throws {
        let apiIPAddress = "192.168.105.1"
        let apiPort = "80"
        XCTAssertFalse(
            try canConnectSocket(host: apiIPAddress, port: apiPort),
            "Failed to verify that local network cannot be accessed"
        )
    }
}
