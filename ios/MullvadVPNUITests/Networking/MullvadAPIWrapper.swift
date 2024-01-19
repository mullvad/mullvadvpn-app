//
//  AppAPI.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

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

    public static func getAPIIPAddress() -> String? {
        guard let ipAddress = endpoint.components(separatedBy: ":").first else {
            XCTFail("Endpoint value is not in the format <IP-address>:<port>")
            return nil
        }

        return ipAddress
    }

    public static func getAPIPort() -> String? {
        guard let port = endpoint.components(separatedBy: ":").last else {
            XCTFail("Endpoint value is not in the format <IP-address>:<port>")
            return nil
        }

        return port
    }
}
