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

/// Whether DAITA is enabled
public enum DAITAState: Codable, Sendable {
    case on
    case off

    public var isEnabled: Bool {
        self == .on
    }
}

/// Whether "direct only" is enabled, meaning no automatic routing to DAITA relays.
public enum DirectOnlyState: Codable, Sendable {
    case on
    case off

    public var isEnabled: Bool {
        self == .on
    }
}

/// Selected relay is incompatible with DAITA, either through singlehop or multihop.
public enum DAITASettingsCompatibilityError {
    case singlehop, multihop
}

public struct DAITASettings: Codable, Equatable, Sendable {
    private enum CodingKeys: String, CodingKey {
        case state
        case daitaState
        case directOnlyState
    }

    // We should use @diagnose when we move to Swift 6.4. It will make handling warnings easier.
    @available(*, deprecated, renamed: "daitaState")
    public let state: DAITAState = .off

    public var daitaState: DAITAState
    private var _directOnlyState: DirectOnlyState

    @available(
        *, deprecated,
        message: "Deprecated: Do not use in new implementations. It is supported only for multihop migration."
    )
    public var directOnlyState: DirectOnlyState {
        _directOnlyState
    }

    public var isEnabled: Bool {
        get { daitaState == .on }
        set { daitaState = newValue ? .on : .off }
    }

    /// Whether the legacy direct-only mode was enabled. For use in multihop migration only.
    public var isDirectOnly: Bool {
        _directOnlyState.isEnabled
    }

    public init(daitaState: DAITAState = .off, directOnlyState: DirectOnlyState = .off) {
        self.daitaState = daitaState
        self._directOnlyState = directOnlyState
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        daitaState =
            try container.decodeIfPresent(DAITAState.self, forKey: .daitaState)
            ?? container.decodeIfPresent(DAITAState.self, forKey: .state)
            ?? .off

        _directOnlyState =
            try container.decodeIfPresent(
                DirectOnlyState.self,
                forKey: .directOnlyState
            )
            ?? .off
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(daitaState, forKey: .daitaState)
        try container.encode(_directOnlyState, forKey: .directOnlyState)
    }
}
