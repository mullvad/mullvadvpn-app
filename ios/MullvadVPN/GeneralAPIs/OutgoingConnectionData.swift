// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Network

typealias IPV4ConnectionData = OutgoingConnectionData<IPv4Address>
typealias IPV6ConnectionData = OutgoingConnectionData<IPv6Address>

// MARK: - OutgoingConnectionData

struct OutgoingConnectionData<T: Codable & IPAddress>: Codable, Equatable {
    let ip: T
    let exitIP: Bool

    enum CodingKeys: String, CodingKey {
        case ip
        case exitIP = "mullvad_exit_ip"
    }

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.ip.rawValue == rhs.ip.rawValue && lhs.exitIP == rhs.exitIP
    }
}
