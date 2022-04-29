//
//  ApplicationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address

enum ApplicationConfiguration {}

extension ApplicationConfiguration {
    /// The application group identifier used for sharing application preferences between processes
    static let securityGroupIdentifier = "group.net.mullvad.MullvadVPN"

    /// The application identifier for the PacketTunnel extension
    static let packetTunnelExtensionIdentifier = "net.mullvad.MullvadVPN.PacketTunnel"

    /// Container URL for security group
    static var containerURL: URL? {
        return FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: Self.securityGroupIdentifier)
    }

    /// The main application log file located in a shared container
    static var mainApplicationLogFileURL: URL? {
        return Self.containerURL?.appendingPathComponent("Logs/net.mullvad.MullvadVPN.log", isDirectory: false)
    }

    /// The packet tunnel log file located in a shared container
    static var packetTunnelLogFileURL: URL? {
        return Self.containerURL?.appendingPathComponent("Logs/net.mullvad.MullvadVPN.PacketTunnel.log", isDirectory: false)
    }

    /// All log files located in a shared container
    static var logFileURLs: [URL] {
        return [mainApplicationLogFileURL, packetTunnelLogFileURL].compactMap { $0 }
    }

    /// Privacy policy URL.
    static let privacyPolicyURL = URL(string: "https://mullvad.net/help/privacy-policy/")!

    /// FAQ & Guides URL.
    static let faqAndGuidesURL = URL(string: "https://mullvad.net/help/tag/mullvad-app/")!

    /// Default API hostname.
    static let defaultAPIHostname = "api.mullvad.net"

    /// Default API endpoint.
    static let defaultAPIEndpoint = AnyIPEndpoint(string: "193.138.218.78:443")!

    /// Default network timeout for API requests.
    static let defaultAPINetworkTimeout: TimeInterval = 10

    /// Background fetch minimum interval
    static let minimumBackgroundFetchInterval: TimeInterval = 3600

    /// App refresh background task identifier
    static let appRefreshTaskIdentifier = "net.mullvad.MullvadVPN.AppRefresh"

    /// Key rotation background task identifier
    static let privateKeyRotationTaskIdentifier = "net.mullvad.MullvadVPN.PrivateKeyRotation"

    /// API address background task identifier
    static let addressCacheUpdateTaskIdentifier = "net.mullvad.MullvadVPN.AddressCacheUpdate"
}
