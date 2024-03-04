//
//  AccountMock.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-03-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

extension Account {
    static func mock(expiry: Date = .distantFuture) -> Account {
        Account(
            id: "account-id",
            expiry: expiry,
            maxDevices: 5,
            canAddDevices: true
        )
    }
}
