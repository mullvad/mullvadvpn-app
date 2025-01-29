//
//  SwiftConnectionModeProvider.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2025-02-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol SwiftConnectionModeProviding {
    func initial()
    func pickMethod()
    func rotate()
}

public class SwiftConnectionModeProvider: SwiftConnectionModeProviding {
    let provider: SwiftConnectionModeProviding

    init(provider: SwiftConnectionModeProviding) {
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
