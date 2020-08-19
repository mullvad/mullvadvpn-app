//
//  ApplicationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

class ApplicationConfiguration {

    /// The application group identifier used for sharing application preferences between processes
    static let securityGroupIdentifier = "group.net.mullvad.MullvadVPN"

    /// The application identifier for the PacketTunnel extension
    static let packetTunnelExtensionIdentifier = "net.mullvad.MullvadVPN.PacketTunnel"

    /// The application log files
    static var logFileURLs: [URL] {
        let containerURL = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: Self.securityGroupIdentifier)
        let fileNames = ["net.mullvad.MullvadVPN", "net.mullvad.MullvadVPN.PacketTunnel"]

        return fileNames.compactMap { (fileName) -> URL? in
            return containerURL?
                .appendingPathComponent("Logs", isDirectory: true)
                .appendingPathComponent(fileName, isDirectory: false)
                .appendingPathExtension("log")
        }
    }
}
