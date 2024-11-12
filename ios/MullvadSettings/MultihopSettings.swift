//
//  MultihopSettings.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-04-26.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Whether Multihop is enabled
public enum MultihopState: Codable {
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
