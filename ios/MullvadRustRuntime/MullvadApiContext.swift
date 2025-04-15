//
//  MullvadApiContext.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public struct MullvadApiContext: Sendable {
    enum MullvadApiContextError: Error {
        case failedToConstructApiClient
    }

    public let context: SwiftApiContext

    public init(host: String, address: AnyIPEndpoint, disable_tls: Bool = false) throws {
        context = switch disable_tls {
        case true:
            mullvad_api_init_new_tls_disabled(host, address.description)
        case false:
            mullvad_api_init_new(host, address.description)
        }

        if context._0 == nil {
            throw MullvadApiContextError.failedToConstructApiClient
        }
    }
}

extension SwiftApiContext: @unchecked @retroactive Sendable {}
