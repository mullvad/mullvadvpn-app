//
//  MullvadAddressCacheProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-05-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public func iniSwiftAddressCacheWrapper(provider: DefaultAddressCacheProvider) -> SwiftAddressCacheWrapper {
    let rawProvider = Unmanaged.passUnretained(provider).toOpaque()
    return init_swift_address_cache_wrapper(rawProvider)
}

@_cdecl("swift_get_cached_endpoint")
func getCacheEndpoint(rawAddressCacheProvider: UnsafeMutableRawPointer) -> LateStringDeallocator {
    let addressCacheProvider = Unmanaged<DefaultAddressCacheProvider>.fromOpaque(rawAddressCacheProvider)
        .takeUnretainedValue()
    let cStr = addressCacheProvider.getCurrentEndpoint().description.toCStringPointer()
    return LateStringDeallocator(ptr: cStr, deallocate_ptr: deallocate_pointer(pointer:))
}

func deallocate_pointer(pointer: UnsafePointer<CChar>?) {
    pointer?.deallocate()
}
