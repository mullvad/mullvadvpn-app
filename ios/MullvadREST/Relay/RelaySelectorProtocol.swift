// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes

/// Protocol describing a type that can select a relay.
public protocol RelaySelectorProtocol: Sendable {
    var relayCache: RelayCacheProtocol { get }

    func selectRelays(
        tunnelSettings: LatestTunnelSettings,
        connectionAttemptCount: UInt
    ) throws -> SelectedRelays

    func findCandidates(
        tunnelSettings: LatestTunnelSettings,
        includeInactive: Bool
    ) throws -> RelayCandidates
}

/// Struct describing the selected relay.
public struct SelectedRelay: Equatable, Codable, Sendable {
    /// Selected relay endpoint with resolved socket address and obfuscation.
    public let endpoint: SelectedEndpoint

    /// Relay hostname.
    public let hostname: String

    /// Relay geo location.
    public let location: Location

    public let isIPOverridden: Bool

    /// Relay features, such as `DAITA` or `QUIC`.
    public let features: REST.ServerRelay.Features?

    /// Designated initializer.
    public init(
        endpoint: SelectedEndpoint,
        hostname: String,
        location: Location,
        isIPOverridden: Bool = false,
        features: REST.ServerRelay.Features?
    ) {
        self.endpoint = endpoint
        self.hostname = hostname
        self.location = location
        self.features = features
        self.isIPOverridden = isIPOverridden
    }
}

extension SelectedRelay: CustomDebugStringConvertible {
    public var debugDescription: String {
        "\(hostname)\(isIPOverridden ? " [IP Overridden]" : "")"
    }
}

public struct SelectedRelays: Equatable, Codable, Sendable {
    public let entry: SelectedRelay?
    public let exit: SelectedRelay
    public let retryAttempt: UInt

    public var ingress: SelectedRelay {
        entry ?? exit
    }

    /// The obfuscation method, accessed from the ingress relay's endpoint.
    public var obfuscation: ObfuscationMethod {
        ingress.endpoint.obfuscation
    }

    public init(
        entry: SelectedRelay?,
        exit: SelectedRelay,
        retryAttempt: UInt
    ) {
        self.entry = entry
        self.exit = exit
        self.retryAttempt = retryAttempt
    }
}
