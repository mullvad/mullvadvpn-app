//
//  DAITASettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Whether DAITA is enabled
public enum DAITAState: Codable {
    case on
    case off

    public var isEnabled: Bool {
        self == .on
    }
}

public enum DAITARelayFallbackState: Codable {
    case on
    case off

    public var isEnabled: Bool {
        self == .on
    }
}

public struct DAITASettings: Codable, Equatable {
    public let state: DAITAState
    public let fallbackState: DAITARelayFallbackState

    public init(state: DAITAState = .off, fallbackState: DAITARelayFallbackState = .on) {
        self.state = state
        self.fallbackState = fallbackState
    }
}
