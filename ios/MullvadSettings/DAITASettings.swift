//
//  DAITASettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Whether DAITA is enabled.
public enum DAITAState: Codable, Sendable {
    case on
    case off
}

/// Whether "direct only" is enabled, meaning no automatic routing to DAITA relays.
public enum DirectOnlyState: Codable, Sendable {
    case on
    case off

    public var isEnabled: Bool {
        get {
            self == .on
        }
        set {
            self = newValue ? .on : .off
        }
    }
}

/// Selected relay is incompatible with DAITA, either through singlehop or multihop.
public enum DAITASettingsCompatibilityError {
    case singlehop, multihop
}

public struct DAITASettings: Codable, Equatable, Sendable {
    @available(*, deprecated, renamed: "daitaState")
    public let state: DAITAState = .off

    public var daitaState: DAITAState
    /// > Warning: Deprecated and should only be used in multihop migration.
    public var directOnlyState: DirectOnlyState = .off

    public var isEnabled: Bool {
        get { daitaState == .on }
        set { daitaState = newValue ? .on : .off }
    }

    public init(daitaState: DAITAState = .off) {
        self.daitaState = daitaState
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        daitaState =
            try container.decodeIfPresent(DAITAState.self, forKey: .daitaState)
            ?? container.decodeIfPresent(DAITAState.self, forKey: .state)
            ?? .off
    }
}
