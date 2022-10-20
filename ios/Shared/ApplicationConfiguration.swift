//
//  ApplicationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address

class ApplicationConfiguration {
    /// Shared container security group identifier.
    static var securityGroupIdentifier: String {
        let securityGroupIdentifier = Bundle(for: Self.self)
            .object(forInfoDictionaryKey: "ApplicationSecurityGroupIdentifier") as? String
        return securityGroupIdentifier!
    }

    /// The application identifier for packet tunnel extension.
    static var packetTunnelExtensionIdentifier: String {
        let mainBundleIdentifier = Bundle.main.bundleIdentifier!

        return "\(mainBundleIdentifier).PacketTunnel"
    }

    /// Container URL for security group
    static var containerURL: URL? {
        return FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: Self.securityGroupIdentifier)
    }

    /// The main application log file located in a shared container
    static var mainApplicationLogFileURL: URL? {
        return Self.containerURL?.appendingPathComponent(
            "Logs/net.mullvad.MullvadVPN.log",
            isDirectory: false
        )
    }

    /// The packet tunnel log file located in a shared container
    static var packetTunnelLogFileURL: URL? {
        return Self.containerURL?.appendingPathComponent(
            "Logs/net.mullvad.MullvadVPN.PacketTunnel.log",
            isDirectory: false
        )
    }

    /// All log files located in a shared container
    static var logFileURLs: [URL] {
        return [mainApplicationLogFileURL, packetTunnelLogFileURL].compactMap { $0 }
    }

    /// Privacy policy URL.
    static let privacyPolicyURL = URL(string: "https://mullvad.net/help/privacy-policy/")!

    /// FAQ & Guides URL.
    static let faqAndGuidesURL = URL(string: "https://mullvad.net/help/tag/mullvad-app/")!

    /// Maximum number of devices per account.
    static let maxAllowedDevices = 5

    /// App refresh background task identifier
    static let appRefreshTaskIdentifier = "net.mullvad.MullvadVPN.AppRefresh"

    /// Key rotation background task identifier
    static let privateKeyRotationTaskIdentifier = "net.mullvad.MullvadVPN.PrivateKeyRotation"

    /// API address background task identifier
    static let addressCacheUpdateTaskIdentifier = "net.mullvad.MullvadVPN.AddressCacheUpdate"

    private init() {}
}
