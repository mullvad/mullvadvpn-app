//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

func onAccessChangeCallback(selfPtr: UnsafeRawPointer?, bytes: UnsafePointer<UInt8>?) {
    guard let selfPtr, let bytes else { return }
    let context = Unmanaged<MullvadApiContext>.fromOpaque(selfPtr).takeUnretainedValue()

    let uuid = NSUUID(uuidBytes: bytes) as UUID
    context.accessMethodChangeListeners.forEach { $0.accessMethodChangedTo(uuid) }
}

public class MullvadApiContext: @unchecked Sendable {
    enum Error: Swift.Error {
        case failedToConstructApiClient
    }

    public private(set) var context: SwiftApiContext!
    private let shadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding!
    private let shadowsocksBridgeProviderWrapper: SwiftShadowsocksLoaderWrapper!
    public let accessMethodChangeListeners: [MullvadAccessMethodChangeListening]

    public init(
        host: String,
        address: String,
        domain: String,
        disableTls: Bool = false,
        shadowsocksProvider: SwiftShadowsocksBridgeProviding,
        accessMethodWrapper: SwiftAccessMethodSettingsWrapper,
        accessMethodChangeListeners: [MullvadAccessMethodChangeListening]
    ) throws {
        let bridgeProvider = SwiftShadowsocksBridgeProvider(provider: shadowsocksProvider)
        self.shadowsocksBridgeProvider = bridgeProvider
        self.shadowsocksBridgeProviderWrapper = initMullvadShadowsocksBridgeProvider(provider: bridgeProvider)

        self.accessMethodChangeListeners = accessMethodChangeListeners

        let selfPtr = Unmanaged.passUnretained(self).toOpaque()
        context =
            switch disableTls {
            case true:
                mullvad_api_init_new_tls_disabled(
                    host,
                    address,
                    domain,
                    shadowsocksBridgeProviderWrapper,
                    accessMethodWrapper,
                    onAccessChangeCallback,
                    selfPtr
                )
            case false:
                mullvad_api_init_new(
                    host,
                    address,
                    domain,
                    shadowsocksBridgeProviderWrapper,
                    accessMethodWrapper,
                    onAccessChangeCallback,
                    selfPtr
                )
            }

        if context._0 == nil {
            throw Error.failedToConstructApiClient
        }
    }
}

extension SwiftApiContext: @unchecked @retroactive Sendable {}
