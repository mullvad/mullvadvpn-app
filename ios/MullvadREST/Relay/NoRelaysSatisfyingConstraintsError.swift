//
//  NoRelaysSatisfyingConstraintsError.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum NoRelaysSatisfyingConstraintsReason {
    case filterConstraintNotMatching
    case invalidPort
    case multihopEntryEqualsExit
    case multihopOther
    case noActiveRelaysFound
    case noDaitaRelaysFound
    case relayConstraintNotMatching
}

public struct NoRelaysSatisfyingConstraintsError: LocalizedError {
    public let reason: NoRelaysSatisfyingConstraintsReason

    public var errorDescription: String? {
        "No relays satisfying constraints."
    }

    public init(_ reason: NoRelaysSatisfyingConstraintsReason) {
        self.reason = reason
    }
}
