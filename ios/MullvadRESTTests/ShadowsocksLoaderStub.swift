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
    private let _configuration: ShadowsocksConfiguration
    init(configuration: ShadowsocksConfiguration) {
        _configuration = configuration
    }

    var configuration: ShadowsocksConfiguration {
        _configuration
    }

    func preloadNewConfiguration() throws {}
}
