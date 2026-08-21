// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
