//
//  MullvadAPIWrapper.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import CryptoKit
import Foundation
import XCTest

enum MullvadAPIError: Error {
    case invalidEndpointFormatError
    case requestError
}

class MullvadAPIWrapper {
    private var mullvadAPI: MullvadApi
    private let throttleQueue = DispatchQueue(label: "MullvadAPIWrapperThrottleQueue", qos: .userInitiated)
    private var lastCallDate: Date?
    private let throttleDelay: TimeInterval = 0.25
    private let throttleWaitTimeout: TimeInterval = 5.0

    // swiftlint:disable force_cast
    static let hostName = Bundle(for: MullvadAPIWrapper.self)
        .infoDictionary?["ApiHostName"] as! String

    /// API endpoint configuration value in the format <IP-address>:<port>
    static let endpoint = Bundle(for: MullvadAPIWrapper.self)
        .infoDictionary?["ApiEndpoint"] as! String
    // swiftlint:enable force_cast

    init() throws {
        let apiAddress = try Self.getAPIIPAddress() + ":" + Self.getAPIPort()
        let hostname = Self.hostName
        mullvadAPI = try MullvadApi(apiAddress: apiAddress, hostname: hostname)
    }

    /// Throttle what's in the callback. This is used for throttling requests to the app API. All requests should be throttled or else we might be rate limited. The API allows maximum 5 requests per second. Note that the implementation assumes what is being throttled is synchronous.
    private func throttle(callback: @escaping () -> Void) {
        throttleQueue.async {
            let now = Date()
            var delay: TimeInterval = 0

            if let lastCallDate = self.lastCallDate {
                let timeSinceLastCall = now.timeIntervalSince(lastCallDate)

                if timeSinceLastCall < self.throttleDelay {
                    delay = self.throttleDelay - timeSinceLastCall
                }
            }

            // Note that this is not really throttling, but it works because the API client implementation is synchronous, and we wait for each request to return.
            self.throttleQueue.asyncAfter(deadline: .now() + delay) {
                callback()
                self.lastCallDate = Date()
            }
        }
    }

    public static func getAPIIPAddress() throws -> String {
        guard let ipAddress = endpoint.components(separatedBy: ":").first else {
            throw MullvadAPIError.invalidEndpointFormatError
        }

        return ipAddress
    }

    public static func getAPIPort() throws -> String {
        guard let port = endpoint.components(separatedBy: ":").last else {
            throw MullvadAPIError.invalidEndpointFormatError
        }

        return port
    }

    /// Generate a mock public WireGuard key
    private func generateMockWireGuardKey() -> Data {
        let privateKey = Curve25519.KeyAgreement.PrivateKey()
        let publicKey = privateKey.publicKey
        let publicKeyData = publicKey.rawRepresentation

        return publicKeyData
    }

    func createAccount() -> String {
        var accountNumber = String()
        let requestCompletedExpectation = XCTestExpectation(description: "Create account request completed")

        throttle {
            do {
                accountNumber = try self.mullvadAPI.createAccount()
            } catch {
                XCTFail("Failed to create account with error: \(error.localizedDescription)")
            }

            requestCompletedExpectation.fulfill()
        }

        let waitResult = XCTWaiter().wait(for: [requestCompletedExpectation], timeout: throttleWaitTimeout)
        XCTAssertEqual(waitResult, .completed, "Create account request completed")

        return accountNumber
    }

    func deleteAccount(_ accountNumber: String) {
        let requestCompletedExpectation = XCTestExpectation(description: "Delete account request completed")

        throttle {
            do {
                try self.mullvadAPI.delete(account: accountNumber)
            } catch {
                XCTFail("Failed to delete account with error: \(error.localizedDescription)")
            }

            requestCompletedExpectation.fulfill()
        }

        let waitResult = XCTWaiter().wait(for: [requestCompletedExpectation], timeout: throttleWaitTimeout)
        XCTAssertEqual(waitResult, .completed, "Delete account request completed")
    }

    /// Add another device to specified account. A dummy WireGuard key will be generated.
    func addDevice(_ account: String) {
        let requestCompletedExpectation = XCTestExpectation(description: "Add device request completed")

        throttle {
            let devicePublicKey = self.generateMockWireGuardKey()

            do {
                try self.mullvadAPI.addDevice(forAccount: account, publicKey: devicePublicKey)
            } catch {
                XCTFail("Failed to add device with error: \(error.localizedDescription)")
            }

            requestCompletedExpectation.fulfill()
        }

        let waitResult = XCTWaiter().wait(for: [requestCompletedExpectation], timeout: throttleWaitTimeout)
        XCTAssertEqual(waitResult, .completed, "Add device request completed")
    }

    /// Add multiple devices to specified account. Dummy WireGuard keys will be generated.
    func addDevices(_ numberOfDevices: Int, account: String) {
        for i in 0 ..< numberOfDevices {
            self.addDevice(account)
            print("Created \(i + 1) devices")
        }
    }

    func getAccountExpiry(_ account: String) throws -> Date {
        var accountExpiryDate: Date = .distantPast
        let requestCompletedExpectation = XCTestExpectation(description: "Get account expiry request completed")

        throttle {
            do {
                let accountExpiryTimestamp = Double(try self.mullvadAPI.getExpiry(forAccount: account))
                accountExpiryDate = Date(timeIntervalSince1970: accountExpiryTimestamp)
            } catch {
                XCTFail("Failed to get account expiry with error: \(error.localizedDescription)")
            }

            requestCompletedExpectation.fulfill()
        }

        let waitResult = XCTWaiter().wait(for: [requestCompletedExpectation], timeout: throttleWaitTimeout)
        XCTAssertEqual(waitResult, .completed, "Get account expiry request completed")

        return accountExpiryDate
    }

    func getDevices(_ account: String) throws -> [Device] {
        var devices: [Device] = []
        let requestCompletedExpectation = XCTestExpectation(description: "Get devices request completed")

        throttle {
            do {
                devices = try self.mullvadAPI.listDevices(forAccount: account)
            } catch {
                XCTFail("Failed to get devices with error: \(error.localizedDescription)")
            }

            requestCompletedExpectation.fulfill()
        }

        let waitResult = XCTWaiter.wait(for: [requestCompletedExpectation], timeout: throttleWaitTimeout)
        XCTAssertEqual(waitResult, .completed, "Get devices request completed")

        return devices
    }
}
