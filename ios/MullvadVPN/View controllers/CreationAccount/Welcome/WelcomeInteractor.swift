//
//  WelcomeInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
class WelcomeInteractor {
    private let deviceData: StoredDeviceData
    private let accountData: StoredAccountData

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: deviceData.capitalizedName,
            accountNumber: accountData.number.formattedAccountNumber
        )
    }

    init(deviceData: StoredDeviceData, accountData: StoredAccountData) {
        self.deviceData = deviceData
        self.accountData = accountData
    }
}
