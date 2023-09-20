//
//  RootConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-19.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

struct RootDeviceInfoViewModel {
    let configuration: RootConfiguration
    init(isPresentingAccountExpiryBanner: Bool, deviceState: DeviceState) {
        configuration = RootConfiguration(
            deviceName: deviceState.deviceData?.capitalizedName,
            expiry: (isPresentingAccountExpiryBanner || (deviceState.accountData?.isExpired ?? true))
                ? nil
                : deviceState.accountData?.expiry,
            showsAccountButton: deviceState.isLoggedIn
        )
    }
}

struct RootConfiguration {
    var deviceName: String?
    var expiry: Date?
    var showsAccountButton: Bool
}
