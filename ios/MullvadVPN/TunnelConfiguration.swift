//
//  TunnelConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import NetworkExtension

/// A struct that holds a tun interface configuration
struct InterfaceConfiguration: Codable {
    var privateKey = WireguardPrivateKey()
    var addresses = [IPAddressRange]()
}

/// A struct that holds the configuration passed via NETunnelProviderProtocol
struct TunnelConfiguration: Codable {
    var relayConstraints = RelayConstraints()
    var interface = InterfaceConfiguration()
}
