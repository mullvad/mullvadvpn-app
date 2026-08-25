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
import MullvadTypes

/// A protocol that formalizes remote service dependency used by `DeviceCheckOperation`.
protocol DeviceCheckRemoteServiceProtocol: Sendable {
    func getAccountData(accountNumber: String) async -> Result<Account, Error>

    func getDevice(
        accountNumber: String,
        identifier: String
    ) async -> Result<Device, Error>

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey
    ) async -> Result<Device, Error>
}
