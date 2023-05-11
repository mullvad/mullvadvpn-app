//
//  RootConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-19.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct RootConfiguration {
    var deviceName: String?
    var expiry: Date?
    var showsAccountButton: Bool
    let showsDeviceInfo: Bool
}
