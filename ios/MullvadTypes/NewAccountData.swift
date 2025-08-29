//
//  NewAccountData.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-04-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct NewAccountData: Decodable, Sendable {
    public let id: String
    public let expiry: Date
    public let maxPorts: Int
    public let canAddPorts: Bool
    public let maxDevices: Int
    public let canAddDevices: Bool
    public let number: String

    public init(
        id: String,
        expiry: Date,
        maxPorts: Int,
        canAddPorts: Bool,
        maxDevices: Int,
        canAddDevices: Bool,
        number: String
    ) {
        self.id = id
        self.expiry = expiry
        self.maxPorts = maxPorts
        self.canAddPorts = canAddPorts
        self.maxDevices = maxDevices
        self.canAddDevices = canAddDevices
        self.number = number
    }
}
