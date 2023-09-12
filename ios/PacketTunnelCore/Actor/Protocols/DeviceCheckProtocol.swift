//
//  DeviceCheckProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 12/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol DeviceCheckProtocol {
    func start(rotateKeyOnMismatch: Bool) async throws -> DeviceCheck
}
