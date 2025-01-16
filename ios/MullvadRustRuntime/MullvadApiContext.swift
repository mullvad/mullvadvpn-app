//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public struct MullvadApiContext: Sendable {
    public let context: SwiftApiContext

    public init(host: String, address: String) throws {
        context = mullvad_api_init_new(host, address)

        if context._0 == nil {
            throw NSError(domain: "", code: 0)
        }
    }
}

extension SwiftApiContext: @unchecked @retroactive Sendable {}
