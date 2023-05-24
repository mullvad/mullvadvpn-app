//
//  AccountData.swift
//  MullvadTypes
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Type holding account related data returned from REST API.
public struct AccountData: Codable, Equatable {
    public let id: String
    public let expiry: Date
    public let maxPorts: Int
    public let canAddPorts: Bool
    public let maxDevices: Int
    public let canAddDevices: Bool
}
