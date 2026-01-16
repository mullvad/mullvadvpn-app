//
//  InclueAllNetworksSettings.swift
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

public struct InclueAllNetworksSettings: Codable, Equatable, Sendable {
    public var includeAllNetworksState: InclueAllNetworksState
    public var localNetworkSharingState: LocalNetworkSharingState
    public var consent: Bool

    public var includeAllNetworksIsEnabled: Bool {
        includeAllNetworksState.isEnabled && consent
    }

    public var localNetworkSharingIsEnabled: Bool {
        includeAllNetworksState.isEnabled && localNetworkSharingState.isEnabled && consent
    }

    public init(
        includeAllNetworksState: InclueAllNetworksState = .off,
        localNetworkSharingState: LocalNetworkSharingState = .off,
        consent: Bool = false
    ) {
        self.includeAllNetworksState = includeAllNetworksState
        self.localNetworkSharingState = localNetworkSharingState
        self.consent = consent
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.includeAllNetworksState = try container.decode(InclueAllNetworksState.self, forKey: .includeAllNetworksState)
        self.localNetworkSharingState = try container.decode(LocalNetworkSharingState.self, forKey: .localNetworkSharingState)
        self.consent = try container.decode(Bool.self, forKey: .consent)
    }
}
