//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes

func onAccessChangeCallback(selfPtr: UnsafeRawPointer?, bytes: UnsafePointer<UInt8>?) {
    guard let selfPtr, let bytes else { return }
    let context = selfPtr.assumingMemoryBound(to: MullvadApiContext.self).pointee

    let uuid = NSUUID(uuidBytes: bytes) as UUID
    context.accessMethodChangeListener?.accessMethodChangedTo(uuid)
}

public class MullvadApiContext: @unchecked Sendable {
    enum MullvadApiContextError: Error {
        case failedToConstructApiClient
    }

    public private(set) var context: SwiftApiContext!
    private let shadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding!
    private let shadowsocksBridgeProviderWrapper: SwiftShadowsocksLoaderWrapper!
    private let addressCacheWrapper: SwiftAddressCacheWrapper!
    private let addressCacheProvider: AddressCacheProviding!
    public var accessMethodChangeListener: MullvadAccessMethodChangeListening?

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

        context = nil
        context = switch disableTls {
        case true:
            mullvad_api_init_new_tls_disabled(
                host,
                address,
                domain,
                shadowsocksBridgeProviderWrapper,
                accessMethodWrapper,
                addressCacheWrapper,
                onAccessChangeCallback,
                Unmanaged.passRetained(self).toOpaque()
            )
        case false:
            mullvad_api_init_new(
                host,
                address,
                domain,
                shadowsocksBridgeProviderWrapper,
                accessMethodWrapper,
                addressCacheWrapper,
                onAccessChangeCallback,
                Unmanaged.passRetained(self).toOpaque()
            )
        }

        if context._0 == nil {
            throw MullvadApiContextError.failedToConstructApiClient
        }
    }
}

extension SwiftApiContext: @unchecked @retroactive Sendable {}
