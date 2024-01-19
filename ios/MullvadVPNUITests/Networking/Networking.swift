//
//  Networking.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import XCTest

enum NetworkingError: Error {
    case notConfiguredError
    case internalError(reason: String)
}

/// Class with methods for verifying network connectivity
class Networking {
    /// Get IP address of the iOS device under test
    static func getIPAddress() throws -> String {
        var ipAddress: String
        // Get list of all interfaces on the local machine:
        var interfaceList: UnsafeMutablePointer<ifaddrs>?
        guard getifaddrs(&interfaceList) == 0, let firstInterfaceAddress = interfaceList else {
            throw NetworkingError.internalError(reason: "Failed to locate local networking interface")
        }

        // For each interface
        for interfacePointer in sequence(first: firstInterfaceAddress, next: { $0.pointee.ifa_next }) {
            let flags = Int32(interfacePointer.pointee.ifa_flags)
            let interfaceAddress = interfacePointer.pointee.ifa_addr.pointee

            // Check for running IPv4 interfaces. Skip the loopback interface.
            if (
                flags &
                    (IFF_UP | IFF_RUNNING | IFF_LOOPBACK)
            ) == (IFF_UP | IFF_RUNNING),
                interfaceAddress.sa_family == UInt8(AF_INET) {
                // Check if interface is en0 which is the WiFi connection on the iPhone
                let name = String(cString: interfacePointer.pointee.ifa_name)
                if name == "en0" {
                    // Convert interface address to a human readable string:
                    var hostname = [CChar](repeating: 0, count: Int(NI_MAXHOST))
                    if getnameinfo(
                        interfacePointer.pointee.ifa_addr,
                        socklen_t(interfaceAddress.sa_len),
                        &hostname,
                        socklen_t(hostname.count),
                        nil,
                        socklen_t(0),
                        NI_NUMERICHOST
                    ) == 0 {
                        ipAddress = String(cString: hostname)
                        return ipAddress
                    }
                }
            }
        }

        freeifaddrs(interfaceList)

        throw NetworkingError.internalError(reason: "Failed to determine device's IP address")
    }

    private static func getAdServingDomainURL() -> URL? {
        guard let adServingDomain = Bundle(for: BaseUITestCase.self)
            .infoDictionary?["AdServingDomain"] as? String,
            let adServingDomainURL = URL(string: adServingDomain) else {
            XCTFail("Ad serving domain not configured")
            return nil
        }

        return adServingDomainURL
    }

    private static func getAdServingDomain() throws -> String {
        guard let adServingDomain = Bundle(for: BaseUITestCase.self)
            .infoDictionary?["AdServingDomain"] as? String else {
            throw NetworkingError.notConfiguredError
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

    /// Verify API can be accessed by attempting to connect a socket to the configured API host and port
    public static func verifyCanAccessAPI() throws {
        let apiIPAddress = try MullvadAPIWrapper.getAPIIPAddress()
        let apiPort = try MullvadAPIWrapper.getAPIPort()
        XCTAssertTrue(canConnectSocket(host: apiIPAddress, port: apiPort))
    }

    /// Verify API cannot be accessed by attempting to connect a socket to the configured API host and port
    public static func verifyCannotAccessAPI() throws {
        let apiIPAddress = try MullvadAPIWrapper.getAPIIPAddress()
        let apiPort = try MullvadAPIWrapper.getAPIPort()
        XCTAssertFalse(canConnectSocket(host: apiIPAddress, port: apiPort))
    }

    /// Verify that an ad serving domain is reachable by making sure a connection can be established on port 80
    public static func verifyCanReachAdServingDomain() {
        XCTAssertTrue(Self.canConnectSocket(host: try Self.getAdServingDomain(), port: "80"))
    }

    /// Verify that an ad serving domain is NOT reachable by making sure a connection can be established on port 80
    public static func verifyCannotReachAdServingDomain() {
        XCTAssertFalse(Self.canConnectSocket(host: try Self.getAdServingDomain(), port: "80"))
    }
}
