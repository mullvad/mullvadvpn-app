//
//  NetworkTester.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import XCTest

/// Class with methods for verifying network connectivity
class NetworkTester {
    private static func getAdServingDomainURL() -> URL {
        guard let adServingDomain = Bundle(for: BaseUITestCase.self)
            .infoDictionary?["AdServingDomain"] as? String,
            let adServingDomainURL = URL(string: adServingDomain) else {
            fatalError("Ad serving domain not configured")
        }

        return adServingDomainURL
    }

    private static func getAdServingDomain() -> String {
        guard let adServingDomain = Bundle(for: BaseUITestCase.self)
            .infoDictionary?["AdServingDomain"] as? String else {
            fatalError("Ad serving domain not configured")
        }

        return adServingDomain
    }

    /// Check whether host and port is reachable by attempting to connect a socket
    private static func canConnectSocket(host: String, port: String) -> Bool {
        let socketHost = NWEndpoint.Host(host)
        let socketPort = NWEndpoint.Port(port)!
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

    /// Check whether HTTP server is reachable by attempting to send a HTTP request
    /// - Returns: `true` if reachable, otherwise `false`
    private static func canReachHTTPServer(url: URL) -> Bool {
        var requestError: Error?
        var requestResponse: URLResponse?

        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the reach ad serving domain request is invoked"
        )

        let task = URLSession.shared.dataTask(with: url) { _, response, error in
            requestError = error
            requestResponse = response
            completionHandlerInvokedExpectation.fulfill()
        }

        task.resume()

        let waitResult = XCTWaiter.wait(for: [completionHandlerInvokedExpectation], timeout: 30)

        if waitResult != .completed,
           let urlError = requestError as? URLError,
           urlError.code == .cannotFindHost && requestResponse == nil {
            return false
        }

        return true
    }

    /// Verify API can be accessed by attempting to connect a socket to the configured API host and port
    public static func verifyCanAccessAPI() {
        let apiIPAddress = MullvadAPIWrapper.getAPIIPAddress()
        let apiPort = MullvadAPIWrapper.getAPIPort()
        XCTAssertTrue(canConnectSocket(host: apiIPAddress, port: apiPort))
    }

    /// Verify API cannot be accessed by attempting to connect a socket to the configured API host and port
    public static func verifyCannotAccessAPI() {
        let apiIPAddress = MullvadAPIWrapper.getAPIIPAddress()
        let apiPort = MullvadAPIWrapper.getAPIPort()
        XCTAssertFalse(canConnectSocket(host: apiIPAddress, port: apiPort))
    }

    /// Verify that an ad serving domain is reachable by making sure a connection can be established on port 80
    public static func verifyCanReachAdServingDomain() {
        XCTAssertTrue(Self.canConnectSocket(host: Self.getAdServingDomain(), port: "80"))
    }

    /// Verify that an ad serving domain is NOT reachable by making sure a connection can be established on port 80
    public static func verifyCannotReachAdServingDomain() {
        XCTAssertFalse(Self.canConnectSocket(host: Self.getAdServingDomain(), port: "80"))
    }
}
