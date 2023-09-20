//
//  DeviceStateAccessorProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 07/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

/// A protocol that formalizes device state accessor dependency used by `DeviceCheckOperation`.
protocol DeviceStateAccessorProtocol {
    func read() throws -> DeviceState
    func write(_ deviceState: DeviceState) throws
}
