//
//  IncludeAllNetworksSettings.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
