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

/// Whether IAN is enabled.
public enum InclueAllNetworksState: Codable, Sendable {
    case on
    case off

    public var isEnabled: Bool {
        get { self == .on }
        set { self = newValue ? .on : .off }
    }
}

/// Whether "Local network sharing" is enabled.
public enum LocalNetworkSharingState: Codable, Sendable {
    case on
    case off

    public var isEnabled: Bool {
        get { self == .on }
        set { self = newValue ? .on : .off }
    }
}

public struct IncludeAllNetworksSettings: Codable, Equatable, Sendable, CustomDebugStringConvertible {
    public var includeAllNetworksState: InclueAllNetworksState
    public var localNetworkSharingState: LocalNetworkSharingState

    public var includeAllNetworksIsEnabled: Bool {
        includeAllNetworksState.isEnabled
    }

    public var localNetworkSharingIsEnabled: Bool {
        includeAllNetworksState.isEnabled && localNetworkSharingState.isEnabled
    }

    public init(
        includeAllNetworksState: InclueAllNetworksState = .off,
        localNetworkSharingState: LocalNetworkSharingState = .off
    ) {
        self.includeAllNetworksState = includeAllNetworksState
        self.localNetworkSharingState = localNetworkSharingState
    }

    public var debugDescription: String {
        "IncludeAllNetworksSettings(includeAllNetworksState: \(includeAllNetworksState), localNetworkSharingState: \(localNetworkSharingState))"
    }
}
