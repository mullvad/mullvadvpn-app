//
//  MullvadAPIWrapper.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

enum MullvadAPIError: Error {
    case incorrectConfigurationFormat
    case requestError
}

class MullvadAPIWrapper {
    // swiftlint:disable force_cast
    static let hostName = Bundle(for: MullvadAPIWrapper.self)
        .infoDictionary?["ApiHostName"] as! String

    private var mullvadAPI: MullvadApi

    /// API endpoint configuration value in the format <IP-address>:<port>
    static let endpoint = Bundle(for: MullvadAPIWrapper.self)
        .infoDictionary?["ApiEndpoint"] as! String
    // swiftlint:enable force_cast

    init() throws {
        let apiAddress = try Self.getAPIIPAddress() + ":" + Self.getAPIPort()
        let hostname = Self.getAPIHostname()
        mullvadAPI = try MullvadApi(apiAddress: apiAddress, hostname: hostname)
    }

    public static func getAPIHostname() -> String {
        return hostName
    }

    public static func getAPIIPAddress() throws -> String {
        guard let ipAddress = endpoint.components(separatedBy: ":").first else {
            throw MullvadAPIError.incorrectConfigurationFormat
        }

        return ipAddress
    }

    public static func getAPIPort() throws -> String {
        guard let port = endpoint.components(separatedBy: ":").last else {
            throw MullvadAPIError.incorrectConfigurationFormat
        }

        return port
    }

    /// Generate a mock WireGuard key
    private func generateMockWireGuardKey() -> Data {
        var bytes = [UInt8]()

        for _ in 0 ..< 44 {
            bytes.append(UInt8.random(in: 0 ..< 255))
        }

        return Data(bytes)
    }

    func createAccount() -> String {
        do {
            let accountNumber = try mullvadAPI.createAccount()
            return accountNumber
        } catch {
            XCTFail("Failed to create account using app API")
            return String()
        }
    }

    func deleteAccount(_ accountNumber: String) {
        do {
            try mullvadAPI.delete(account: accountNumber)
        } catch {
            XCTFail("Failed to delete account using app API")
        }
    }

    func accountExists(_ accountNumber: String) -> Bool {
        do {
            let _ = try mullvadAPI.getExpiry(forAccount: accountNumber)
            return true
        } catch {
            return false
        }
    }

    /// Add another device to specified account. A dummy WireGuard key will be generated.
    func addDevice(_ account: String) throws {
        let devicePublicKey = generateMockWireGuardKey()

        do {
            try mullvadAPI.addDevice(forAccount: account, publicKey: devicePublicKey)
        } catch {
            throw MullvadAPIError.requestError
        }
    }

    func getAccountExpiry(_ account: String) throws -> UInt64 {
        do {
            return try mullvadAPI.getExpiry(forAccount: account)
        } catch {
            throw MullvadAPIError.requestError
        }
    }
}
