//
//  OutgoingConnectionData.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-11-15.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

typealias IPV4ConnectionData = OutgoingConnectionData<IPv4Address>
typealias IPV6ConnectionData = OutgoingConnectionData<IPv6Address>

// MARK: - OutgoingConnectionData

struct OutgoingConnectionData<T: Codable & IPAddress>: Codable, Equatable {
    let ip: T
    let exitIP: Bool

    enum CodingKeys: String, CodingKey {
        case ip, exitIP = "mullvad_exit_ip"
    }

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.ip.rawValue == rhs.ip.rawValue && lhs.exitIP == rhs.exitIP
    }
}
