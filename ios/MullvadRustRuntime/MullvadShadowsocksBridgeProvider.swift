//
//  MullvadShadowsocksBridgeProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-03-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public func initMullvadShadowsocksBridgeProvider(provider: SwiftShadowsocksBridgeProvider)
    -> SwiftShadowsocksLoaderWrapper
{
    let rawProvider = Unmanaged.passUnretained(provider).toOpaque()
    return init_swift_shadowsocks_loader_wrapper(rawProvider)
}
