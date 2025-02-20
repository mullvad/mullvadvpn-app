//
//  SwiftConnectionModeProvider.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2025-02-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol SwiftConnectionModeProviding: Sendable {
    func initial()
    func pickMethod()
    func rotate()
}

public final class SwiftConnectionModeProviderProxy: SwiftConnectionModeProviding, Sendable {
    let provider: SwiftConnectionModeProviding

    public init(provider: SwiftConnectionModeProviding) {
        self.provider = provider
    }

    public func initial() {
        provider.initial()
    }

    public func pickMethod() {
        provider.pickMethod()
    }

    public func rotate() {
        provider.rotate()
    }
}
