//
//  ShadowsocksBridgeProviding.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2025-03-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol SwiftShadowsocksBridgeProviding: Sendable {
    func bridge() -> ShadowsocksConfiguration?
}

public final class SwiftShadowsocksBridgeProvider: SwiftShadowsocksBridgeProviding, Sendable {
    let provider: SwiftShadowsocksBridgeProviding

    public init(provider: SwiftShadowsocksBridgeProviding) {
        self.provider = provider
    }

    public func bridge() -> ShadowsocksConfiguration? {
        provider.bridge()
    }
}
