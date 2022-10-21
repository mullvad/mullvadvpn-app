//
//  RelaySelectorResult+PacketTunnelRelay.swift
//  TunnelProviderMessaging
//
//  Created by pronebird on 20/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import RelaySelector

extension RelaySelectorResult {
    public var packetTunnelRelay: PacketTunnelRelay {
        return PacketTunnelRelay(
            ipv4Relay: endpoint.ipv4Relay,
            ipv6Relay: endpoint.ipv6Relay,
            hostname: relay.hostname,
            location: location
        )
    }
}
