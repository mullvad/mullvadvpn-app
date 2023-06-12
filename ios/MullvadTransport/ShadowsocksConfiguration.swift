//
//  ShadowsocksConfiguration.swift
//  MullvadTransport
//
//  Created by Marco Nikic on 2023-06-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

public struct ShadowsocksConfiguration: Codable {
    public let bridgeAddress: IPv4Address
    public let bridgePort: UInt16
    public let password: String
    public let cipher: String

    public init(bridgeAddress: IPv4Address, bridgePort: UInt16, password: String, cipher: String) {
        self.bridgeAddress = bridgeAddress
        self.bridgePort = bridgePort
        self.password = password
        self.cipher = cipher
    }
}
