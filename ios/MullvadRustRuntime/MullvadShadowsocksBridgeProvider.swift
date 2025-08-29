//
//  MullvadShadowsocksBridgeProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-03-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public func initMullvadShadowsocksBridgeProvider(provider: SwiftShadowsocksBridgeProvider)
    -> SwiftShadowsocksLoaderWrapper {
    let rawProvider = Unmanaged.passUnretained(provider).toOpaque()
    return init_swift_shadowsocks_loader_wrapper(rawProvider)
}

@_cdecl("swift_get_shadowsocks_bridges")
func getShadowsocksBridges(rawBridgeProvider: UnsafeMutableRawPointer) -> UnsafeRawPointer? {
    let bridgeProvider = Unmanaged<SwiftShadowsocksBridgeProvider>.fromOpaque(rawBridgeProvider).takeUnretainedValue()
    guard let bridge = bridgeProvider.bridge() else { return nil }
    let bridgeAddress = bridge.address.rawValue.map { $0 }
    return new_shadowsocks_access_method_setting(
        bridgeAddress,
        UInt(bridgeAddress.count),
        bridge.port,
        bridge.password,
        bridge.cipher
    )
}
