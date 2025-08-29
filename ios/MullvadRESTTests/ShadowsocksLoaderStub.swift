//
//  ShadowsocksLoaderStub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
import MullvadSettings
import MullvadTypes

struct ShadowsocksLoaderStub: ShadowsocksLoaderProtocol, SwiftShadowsocksBridgeProviding {
    func bridge() -> ShadowsocksConfiguration? {
        try? load()
    }

    var configuration: ShadowsocksConfiguration
    var error: Error?

    func clear() throws {
        try load()
    }

    @discardableResult
    func load() throws -> ShadowsocksConfiguration {
        if let error { throw error }
        return configuration
    }
}
