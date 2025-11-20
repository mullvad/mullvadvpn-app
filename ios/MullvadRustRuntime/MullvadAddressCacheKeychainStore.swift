//
//  MullvadAddressCacheKeychainStore.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-11-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

/// Store the address cache, given to us by the Rust code,  to the keychain
@_cdecl("swift_store_address_cache")
func storeAddressCache(_ pointer: UnsafeRawPointer, dataSize: UInt64) {
    let data = Data(bytes: pointer, count: Int(dataSize))
    // TODO: what if this throws?
    try? SettingsManager.writeAddressCache(data)
}

@_cdecl("swift_read_address_cache")
func readAddressCache() -> LateStringDeallocator {
    let data = (try? SettingsManager.readAddressCache()) ?? Data()
    let pointer = UnsafeMutablePointer<CChar>.allocate(capacity: data.count)
    data.withUnsafeBytes { dataPtr in
        pointer.initialize(from: dataPtr, count: data.count)
    }
    return LateStringDeallocator(ptr: pointer, deallocate_ptr: { ptr in ptr?.deallocate() })
}
