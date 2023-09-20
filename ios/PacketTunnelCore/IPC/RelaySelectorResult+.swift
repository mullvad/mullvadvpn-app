//
//  RelaySelectorResult+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 20/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import RelaySelector

extension RelaySelectorResult {
    public var packetTunnelRelay: PacketTunnelRelay {
        PacketTunnelRelay(
            ipv4Relay: endpoint.ipv4Relay,
            ipv6Relay: endpoint.ipv6Relay,
            hostname: relay.hostname,
            location: location
        )
    }
}
