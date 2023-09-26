//
//  PacketTunnelActorProtocol.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol PacketTunnelActorProtocol {
    var state: State { get async }

    func reconnect(to nextRelay: NextRelay)
    func notifyKeyRotation(date: Date?)
}
