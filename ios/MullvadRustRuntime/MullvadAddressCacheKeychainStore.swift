// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings

/// Whether the settings store is available. It requires `ApplicationSecurityGroupIdentifier`
/// to be present in the main bundle's Info.plist, which is not the case in e.g. UI test runners.
private let isSettingsStoreAvailable: Bool =
    Bundle.main
    .object(forInfoDictionaryKey: "ApplicationSecurityGroupIdentifier") != nil

private let settingsManager = SettingsManager()

/// Store the address cache, given to us by the Rust code,  to the keychain
@_cdecl("swift_store_address_cache")
func storeAddressCache(_ pointer: UnsafeRawPointer, dataSize: UInt64) {
    guard isSettingsStoreAvailable else { return }
    let data = Data(bytes: pointer, count: Int(dataSize))
    // if writing to the Keychain fails, it will do so silently.
    try? settingsManager.writeAddressCache(data)
}

@_cdecl("swift_read_address_cache")
func readAddressCache() -> SwiftData {
    guard isSettingsStoreAvailable else { return SwiftData(data: NSData()) }
    let data = (try? settingsManager.readAddressCache()) ?? Data()
    return SwiftData(data: data as NSData)
}
