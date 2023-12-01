//
//  WireGuardObfuscationSettings.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Whether UDP-over-TCP obfuscation is enabled
///
/// `.automatic` means an algorithm will decide whether to use it or not.
public enum WireGuardObfuscationState: Codable {
    case automatic
    case on
    case off
}

/// The port to select when using UDP-over-TCP obfuscation
///
/// `.automatic` means an algorith will decide between using `port80` or `port5001`
public enum WireGuardObfuscationPort: UInt16, Codable {
    case automatic = 0
    case port80 = 80
    case port5001 = 5001

    /// The `UInt16` representation of the port.
    /// - Returns: `0` if `.automatic`, `80` or `5001` otherwise.
    public var portValue: UInt16 {
        self == .automatic ? 0 : rawValue
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

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // Added in 2023.8
        state = try container.decodeIfPresent(WireGuardObfuscationState.self, forKey: .state) ?? .automatic
        port = (try? container.decodeIfPresent(WireGuardObfuscationPort.self, forKey: .port)) ?? .automatic
    }
}
