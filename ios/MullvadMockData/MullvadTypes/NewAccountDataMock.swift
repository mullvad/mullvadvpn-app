//
//  NewAccountDataMock.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-04-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

extension NewAccountData {
    public static func mockValue() -> NewAccountData {
        return NewAccountData(
            id: UUID().uuidString,
            expiry: Date().addingTimeInterval(3600),
            maxPorts: 2,
            canAddPorts: false,
            maxDevices: 5,
            canAddDevices: false,
            number: "1234567890123456"
        )
    }
}
