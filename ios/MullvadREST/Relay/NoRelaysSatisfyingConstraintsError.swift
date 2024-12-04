//
//  NoRelaysSatisfyingConstraintsError.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum NoRelaysSatisfyingConstraintsReason: Sendable {
    case filterConstraintNotMatching
    case invalidPort
    case entryEqualsExit
    case multihopInvalidFlow
    case noActiveRelaysFound
    case noDaitaRelaysFound
    case noObfuscatedRelaysFound
    case relayConstraintNotMatching
}

public struct NoRelaysSatisfyingConstraintsError: LocalizedError, Sendable {
    public let reason: NoRelaysSatisfyingConstraintsReason

    public var errorDescription: String? {
        switch reason {
        case .filterConstraintNotMatching:
            "Filter yields no matching relays"
        case .invalidPort:
            "Invalid port selected by RelaySelector"
        case .entryEqualsExit:
            "Entry and exit relays are the same"
        case .multihopInvalidFlow:
            "Invalid multihop decision flow"
        case .noActiveRelaysFound:
            "No active relays found"
        case .noDaitaRelaysFound:
            "No DAITA relays found"
        case .noObfuscatedRelaysFound:
            "No obfuscated relays found"
        case .relayConstraintNotMatching:
            "Invalid constraint created to pick a relay"
        }
    }

    public init(_ reason: NoRelaysSatisfyingConstraintsReason) {
        self.reason = reason
    }
}
