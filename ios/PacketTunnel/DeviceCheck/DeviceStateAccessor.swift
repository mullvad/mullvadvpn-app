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
import MullvadTypes

/// An object that provides access to `DeviceState` used by `DeviceCheckOperation`.
struct DeviceStateAccessor: DeviceStateAccessorProtocol {
    let settingsManager: SettingsManager

    func read() throws -> DeviceState {
        try settingsManager.readDeviceState()
    }

    func write(_ deviceState: DeviceState) throws {
        try settingsManager.writeDeviceState(deviceState)
    }
}
