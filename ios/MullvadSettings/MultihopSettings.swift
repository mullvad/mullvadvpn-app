//
//  MultihopSettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol MultihopStatePropagation {
    var onNewState: ((MultihopState) -> Void)? { get set }
}

public class MultihopStateUpdater: MultihopStatePropagation {
    public var onNewState: ((MultihopState) -> Void)?

    public init(onNewState: ((MultihopState) -> Void)? = nil) {
        self.onNewState = onNewState
    }
}

/// Whether Multi-hop is enabled
public enum MultihopState: Codable {
    case on
    case off
}

public extension MultihopState {
    var isEnabled: Bool {
        self == .on
    }
}
