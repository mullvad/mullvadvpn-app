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

    func accessMethods() -> [PersistentAccessMethod]
}

public final class SwiftConnectionModeProviderProxy: SwiftConnectionModeProviding, Sendable {
    let provider: SwiftConnectionModeProviding
    public let domainName: String

    public init(provider: SwiftConnectionModeProviding, domainName: String) {
        self.provider = provider
        self.domainName = domainName
    }

    public func accessMethods() -> [PersistentAccessMethod] {
        provider.accessMethods()
    }
}
