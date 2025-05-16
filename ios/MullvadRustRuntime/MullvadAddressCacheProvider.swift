//
//  MullvadAddressCacheProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-05-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public func iniSwiftAddressCacheWrapper(provider: DefaultAddressCacheProvider) -> SwiftAddressCacheWrapper {
    let rawProvider = Unmanaged.passUnretained(provider).toOpaque()
    return init_swift_address_cache_wrapper(rawProvider)
}

@_cdecl("swift_get_cached_endpoint")
func getCacheEndpoint(rawAddressCacheProvider: UnsafeMutableRawPointer) -> UnsafePointer<CChar>! {
    let addressCacheProvider = Unmanaged<DefaultAddressCacheProvider>.fromOpaque(rawAddressCacheProvider)
        .takeUnretainedValue()
    let cStr = addressCacheProvider.getCurrentEndpoint().description.toCStringPointer()
    /**
     `cStr` needs to shortly outlive the return of this function in order to get transformed into a `SocketAddr`
     This is the simplest way to guarantee that the pointer returned does not get deallocated immediately
     Or that no memory is leaked every time this function gets called
     **/
    DispatchQueue(label: "com.MullvadRustRuntime.DecallocateQueue").async {
        cStr?.deallocate()
    }
    return cStr
}
