//
//  DAITASettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Whether DAITA is enabled.
public enum DAITAState: Codable {
    case on
    case off

    public var isEnabled: Bool {
        self == .on
    }
}

/// Whether "direct only" is enabled, meaning no automatic routing to DAITA relays.
public enum DirectOnlyState: Codable {
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

public struct DAITASettings: Codable, Equatable {
    @available(*, deprecated, renamed: "daitaState")
    public let state: DAITAState = .off

    public let daitaState: DAITAState
    public let directOnlyState: DirectOnlyState

    public var shouldDoAutomaticRouting: Bool {
        daitaState.isEnabled && !directOnlyState.isEnabled
    }

    public var shouldDoDirectOnly: Bool {
        daitaState.isEnabled && directOnlyState.isEnabled
    }

    public init(daitaState: DAITAState = .off, directOnlyState: DirectOnlyState = .off) {
        self.daitaState = daitaState
        self.directOnlyState = directOnlyState
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        daitaState = try container.decodeIfPresent(DAITAState.self, forKey: .daitaState)
            ?? container.decodeIfPresent(DAITAState.self, forKey: .state)
            ?? .off

        directOnlyState = try container.decodeIfPresent(DirectOnlyState.self, forKey: .directOnlyState)
            ?? .off
    }
}
