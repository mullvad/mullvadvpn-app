//
//  NoRelaysSatisfyingConstraintsError.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct NoRelaysSatisfyingConstraintsError: LocalizedError {
    public init() {}

    public var errorDescription: String? {
        "No relays satisfying constraints."
    }
}
