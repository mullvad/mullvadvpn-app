//
//  MullvadAddressCacheKeychainStore.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-11-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

/// Whether the settings store is available. It requires `ApplicationSecurityGroupIdentifier`
/// to be present in the main bundle's Info.plist, which is not the case in e.g. UI test runners.
private let isSettingsStoreAvailable: Bool =
    Bundle.main
    .object(forInfoDictionaryKey: "ApplicationSecurityGroupIdentifier") != nil

/// Store the address cache, given to us by the Rust code,  to the keychain
@_cdecl("swift_store_address_cache")
func storeAddressCache(_ pointer: UnsafeRawPointer, dataSize: UInt64) {
    guard isSettingsStoreAvailable else { return }
    let data = Data(bytes: pointer, count: Int(dataSize))
    // if writing to the Keychain fails, it will do so silently.
    try? SettingsManager.writeAddressCache(data)
}

@_cdecl("swift_read_address_cache")
func readAddressCache() -> SwiftData {
    guard isSettingsStoreAvailable else { return SwiftData(data: NSData()) }
    let data = (try? SettingsManager.readAddressCache()) ?? Data()
    return SwiftData(data: data as NSData)
}
