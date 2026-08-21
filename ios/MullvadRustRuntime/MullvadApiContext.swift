//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

class SwiftShadowsocksBridgeProviderWrapper: BridgeProvider, @unchecked Sendable {
    func getBridges() -> ShadowSocksExposed? {
        guard let bridge = self.inner.bridge() else { return nil }
        return ShadowSocksExposed(
            address: bridge.address.rawValue,
            port: bridge.port,
            password: bridge.password,
            cipher: bridge.cipher)
    }

    let inner: SwiftShadowsocksBridgeProviding
    init(inner: SwiftShadowsocksBridgeProviding) {
        self.inner = inner
    }
}

public class MullvadApiContext: @unchecked Sendable, ApiContextCallbackContext, ApiContextCallback {
    public func accessMethodChange(context: any ApiContextCallbackContext, uuid: Data) {
        guard let self = context as? MullvadApiContext else { return }

        // Is this good?
        let parsedUUID = uuid.withUnsafeBytes { (ptr: UnsafeRawBufferPointer) in
            NSUUID(uuidBytes: ptr.baseAddress) as UUID
        }
        self.accessMethodChangeListeners.forEach { $0.accessMethodChangedTo(parsedUUID) }
    }

    enum Error: Swift.Error {
        case failedToConstructApiClient
    }

    public private(set) var context: ApiContext!
    private let shadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding!
    public let accessMethodChangeListeners: [MullvadAccessMethodChangeListening]

    public init(
        host: String,
        address: String,
        domain: String,
        disableTls: Bool = false,
        shadowsocksProvider: SwiftShadowsocksBridgeProviding,
        accessMethodWrapper: SwiftAccessMethodSettingsContext,
        accessMethodChangeListeners: [MullvadAccessMethodChangeListening]
    ) throws {
        let bridgeProvider = SwiftShadowsocksBridgeProvider(provider: shadowsocksProvider)
        self.shadowsocksBridgeProvider = bridgeProvider
        let shadowsocksBridgeProviderWrapper = SwiftShadowsocksBridgeProviderWrapper(inner: shadowsocksBridgeProvider)

        self.accessMethodChangeListeners = accessMethodChangeListeners

        context =
            switch disableTls {
            case true:
                ApiContext.newTlsDisabled(
                    host: host,
                    address: address,
                    domain: domain,
                    bridgeProvider: shadowsocksBridgeProviderWrapper,
                    settingsProvider: accessMethodWrapper,
                    accessMethodChangeCallback: self,
                    accessMethodChangeContext: self)
            case false:
                ApiContext(
                    host: host,
                    address: address,
                    domain: domain,
                    bridgeProvider: shadowsocksBridgeProviderWrapper,
                    settingsProvider: accessMethodWrapper,
                    accessMethodChangeCallback: self,
                    accessMethodChangeContext: self)
            }
    }
}
