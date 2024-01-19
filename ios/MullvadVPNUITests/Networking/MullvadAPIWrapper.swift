//
//  AppAPI.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

enum MullvadAPIError: Error {
    case incorrectConfigurationFormat
}

class MullvadAPIWrapper {
    // swiftlint:disable force_cast
    static let hostName = Bundle(for: MullvadAPIWrapper.self)
        .infoDictionary?["ApiHostName"] as! String

    /// API endpoint configuration value in the format <IP-address>:<port>
    static let endpoint = Bundle(for: MullvadAPIWrapper.self)
        .infoDictionary?["ApiEndpoint"] as! String
    // swiftlint:enable force_cast

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
}
