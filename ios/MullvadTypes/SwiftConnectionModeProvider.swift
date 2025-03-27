//
//  SwiftConnectionModeProvider.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2025-02-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol SwiftConnectionModeProviding: Sendable {
    var domainName: String { get }

    func initial()
    func pickMethod() -> PersistentProxyConfiguration
    func rotate()

    func accessMethods() -> [PersistentAccessMethod]
}

public final class SwiftConnectionModeProviderProxy: SwiftConnectionModeProviding, Sendable {
    let provider: SwiftConnectionModeProviding
    public let domainName: String

    public init(provider: SwiftConnectionModeProviding, domainName: String) {
        self.provider = provider
        self.domainName = domainName
    }

    public func initial() {
        provider.initial()
    }

    public func pickMethod() -> PersistentProxyConfiguration {
        provider.pickMethod()
    }

    public func rotate() {
        provider.rotate()
    }

    public func accessMethods() -> [PersistentAccessMethod] {
        provider.accessMethods()
    }
}
