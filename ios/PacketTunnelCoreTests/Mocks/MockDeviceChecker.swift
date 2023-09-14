//
//  MockDeviceChecker.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore

class MockDeviceChecker: DeviceCheckProtocol {
    func start(rotateKeyOnMismatch: Bool) async throws -> DeviceCheck {
        return DeviceCheck(
            accountVerdict: .active(Account(id: "", expiry: Date(), maxDevices: 5, canAddDevices: true)),
            deviceVerdict: .active,
            keyRotationStatus: .noAction
        )
    }
}
