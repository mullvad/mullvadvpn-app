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
}
