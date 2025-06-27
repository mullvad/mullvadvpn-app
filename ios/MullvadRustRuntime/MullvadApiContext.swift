//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public struct MullvadApiContext: @unchecked Sendable {
    enum MullvadApiContextError: Error {
        case failedToConstructApiClient
    }

    public let context: SwiftApiContext
    private let shadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding!
    private let shadowsocksBridgeProviderWrapper: SwiftShadowsocksLoaderWrapper!
    private let addressCacheWrapper: SwiftAddressCacheWrapper!
    private let addressCacheProvider: AddressCacheProviding!

    public init(
        host: String,
        address: String,
        domain: String,
        disableTls: Bool = false,
        shadowsocksProvider: SwiftShadowsocksBridgeProviding,
        accessMethodWrapper: SwiftAccessMethodSettingsWrapper,
        addressCacheProvider: AddressCacheProviding
    ) throws {
        let bridgeProvider = SwiftShadowsocksBridgeProvider(provider: shadowsocksProvider)
        self.shadowsocksBridgeProvider = bridgeProvider
        self.shadowsocksBridgeProviderWrapper = initMullvadShadowsocksBridgeProvider(provider: bridgeProvider)

        let defaultAddressCache = DefaultAddressCacheProvider(provider: addressCacheProvider)
        self.addressCacheProvider = defaultAddressCache
        self.addressCacheWrapper = iniSwiftAddressCacheWrapper(provider: defaultAddressCache)

        context = switch disableTls {
        case true:
            mullvad_api_init_new_tls_disabled(
                host,
                address,
                domain,
                shadowsocksBridgeProviderWrapper,
                accessMethodWrapper,
                addressCacheWrapper,
                { print("***") }
            )
        case false:
            mullvad_api_init_new(
                host,
                address,
                domain,
                shadowsocksBridgeProviderWrapper,
                accessMethodWrapper,
                addressCacheWrapper,
                { print("***") }
            )
        }

        if context._0 == nil {
            throw MullvadApiContextError.failedToConstructApiClient
        }
    }
}

extension SwiftApiContext: @unchecked @retroactive Sendable {}
