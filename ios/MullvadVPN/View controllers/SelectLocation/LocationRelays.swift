//
//  LocationRelays.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-08-12.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST

struct LocationRelays: Sendable {
    var relays: [REST.ServerRelay]
    var locations: [String: REST.ServerLocation]
}
