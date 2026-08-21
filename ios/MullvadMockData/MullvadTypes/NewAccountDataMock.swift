// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
