//
//  MullvadAPIWrapper.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import CryptoKit
import Foundation
import XCTest

enum MullvadAPIError: Error {
    case invalidEndpointFormatError
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
        let hostname = Self.hostName
        mullvadAPI = try MullvadApi(apiAddress: apiAddress, hostname: hostname)
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

    /// Add another device to specified account. A dummy WireGuard key will be generated.
    func addDevice(_ account: String) throws {
        let devicePublicKey = generateMockWireGuardKey()

        do {
            try mullvadAPI.addDevice(forAccount: account, publicKey: devicePublicKey)
        } catch {
            throw MullvadAPIError.requestError
        }
    }

    /// Add multiple devices to specified account. Dummy WireGuard keys will be generated.
    func addDevices(_ numberOfDevices: Int, account: String) throws {
        for _ in 0 ..< numberOfDevices {
            try self.addDevice(account)
        }
    }

    func getAccountExpiry(_ account: String) throws -> UInt64 {
        do {
            return try mullvadAPI.getExpiry(forAccount: account)
        } catch {
            throw MullvadAPIError.requestError
        }
    }

    func getDevices(_ account: String) throws -> [Device] {
        do {
            return try mullvadAPI.listDevices(forAccount: account)
        } catch {
            throw MullvadAPIError.requestError
        }
    }
}
