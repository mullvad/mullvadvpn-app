// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

public enum NoRelaysSatisfyingConstraintsReason: Sendable {
    case filterConstraintNotMatching
    case invalidPort
    case invalidObfuscationPort
    case entryEqualsExit
    case multihopInvalidFlow
    case noActiveRelaysFound
    case noDaitaRelaysFound
    case noObfuscatedRelaysFound
    case relayConstraintNotMatching
    case noIPv6RelayFound
}

public struct NoRelaysSatisfyingConstraintsError: LocalizedError, Sendable {
    public let reason: NoRelaysSatisfyingConstraintsReason

    public var errorDescription: String? {
        switch reason {
        case .filterConstraintNotMatching:
            "Filter yields no matching relays"
        case .invalidPort:
            "Invalid port selected by RelaySelector"
        case .invalidObfuscationPort:
            "Invalid obfuscation port selected by RelaySelector"
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
        case .noIPv6RelayFound:
            "No relay found that supports IPv6 and all the other constraints"
        }
    }

    public init(_ reason: NoRelaysSatisfyingConstraintsReason) {
        self.reason = reason
    }
}
