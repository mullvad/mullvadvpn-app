//
//  ShadowsocksLoaderStub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

@testable import MullvadREST

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
