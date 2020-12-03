//
//  WireguardAssociatedAddresses.swift
//  MullvadVPN
//
//  Created by pronebird on 13/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import WireGuardKit

struct WireguardAssociatedAddresses: Codable {
    let ipv4Address: IPAddressRange
    let ipv6Address: IPAddressRange
}
