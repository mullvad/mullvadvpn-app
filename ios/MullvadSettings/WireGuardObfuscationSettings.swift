//
//  WireGuardObfuscationSettings.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum WireGuardObfuscationState: Codable {
    case automatic
    case on
    case off
}

public enum WireGuardObfuscationPort: UInt16, Codable {
    case automatic = 0
    case port80 = 80
    case port5001 = 5001

    public var portValue: UInt16 {
        rawValue
    }

    public init?(rawValue: UInt16) {
        switch rawValue {
        case 80:
            self = .port80
        case 5001:
            self = .port5001
        default: self = .automatic
        }
    }
}

public struct WireGuardObfuscationSettings: Codable, Equatable {
    public let state: WireGuardObfuscationState
    public let port: WireGuardObfuscationPort

    public init(state: WireGuardObfuscationState = .automatic, port: WireGuardObfuscationPort = .automatic) {
        self.state = state
        self.port = port
    }
}
