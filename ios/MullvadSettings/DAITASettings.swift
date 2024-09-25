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

/// Whether smart routing is enabled
public enum SmartRoutingState: Codable {
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
    public let daitaState: DAITAState
    public let smartRoutingState: SmartRoutingState

    public init(daitaState: DAITAState = .off, smartRoutingState: SmartRoutingState = .off) {
        self.daitaState = daitaState
        self.smartRoutingState = smartRoutingState
    }
}
