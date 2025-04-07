//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public struct MullvadApiContext: @unchecked Sendable {
    public let context: SwiftApiContext
    private let shadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding!
    private let shadowsocksBridgeProviderWrapper: SwiftShadowsocksLoaderWrapper!

    public init(
        host: String,
        address: String,
        domain: String,
        shadowsocksProvider: SwiftShadowsocksBridgeProviding,
        accessMethodWrapper: SwiftAccessMethodSettingsWrapper
    ) throws {
        let bridgeProvider = SwiftShadowsocksBridgeProvider(provider: shadowsocksProvider)
        self.shadowsocksBridgeProvider = bridgeProvider
        self.shadowsocksBridgeProviderWrapper = initMullvadShadowsocksBridgeProvider(provider: bridgeProvider)

        context = mullvad_api_init_new(
            host,
            address,
            domain,
            shadowsocksBridgeProviderWrapper,
            accessMethodWrapper
        )

        if context._0 == nil {
            throw NSError(domain: "", code: 0)
        }
    }
}

extension SwiftApiContext: @unchecked @retroactive Sendable {}
