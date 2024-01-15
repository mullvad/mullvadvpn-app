//
//  ShadowsocksLoaderStub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
import MullvadSettings
import MullvadTypes

struct ShadowsocksLoaderStub: ShadowsocksLoaderProtocol {
    var configuration: ShadowsocksConfiguration
    var error: Error

    var hasError = false

    init(configuration: ShadowsocksConfiguration, error: Error) {
        self.configuration = configuration
        self.error = error
    }

    func reloadConfiguration() throws {
        try load()
    }

    @discardableResult
    func load() throws -> ShadowsocksConfiguration {
        if hasError {
            throw error
        } else {
            return configuration
        }
    }
}
