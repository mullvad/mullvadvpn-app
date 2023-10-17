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

public enum WireGuardObfuscationPort: Codable {
    case automatic
    case port80
    case port5001
}

public struct WireGuardObfuscationSettings: Codable, Equatable {
    let state: WireGuardObfuscationState
    let port: WireGuardObfuscationPort

    public init(state: WireGuardObfuscationState = .automatic, port: WireGuardObfuscationPort = .automatic) {
        self.state = state
        self.port = port
    }
}
