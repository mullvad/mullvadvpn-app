//
//  WireGuardObfuscationSettings.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Whether obfuscation is enabled and which method is used.
///
/// `.automatic` means an algorithm will decide whether to use obfuscation or not.
public enum WireGuardObfuscationState: Codable {
    @available(*, deprecated, renamed: "udpOverTcp")
    case on

    case automatic
    case udpOverTcp
    case shadowsocks
    case off

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        var allKeys = ArraySlice(container.allKeys)
        guard let key = allKeys.popFirst(), allKeys.isEmpty else {
            throw DecodingError.typeMismatch(
                WireGuardObfuscationState.self,
                DecodingError.Context(
                    codingPath: container.codingPath,
                    debugDescription: "Invalid number of keys found, expected one.",
                    underlyingError: nil
                )
            )
        }

        switch key {
        case .automatic:
            self = .automatic
        case .on, .udpOverTcp:
            self = .udpOverTcp
        case .shadowsocks:
            self = .shadowsocks
        case .off:
            self = .off
        }
    }
}

public enum WireGuardObfuscationUdpOverTcpPort: Codable, Equatable, CustomStringConvertible {
    case automatic
    case port80
    case port5001

    public var portValue: UInt16? {
        switch self {
        case .automatic:
            nil
        case .port80:
            80
        case .port5001:
            5001
        }
    }

    public var description: String {
        switch self {
        case .automatic:
            NSLocalizedString(
                "WIREGUARD_OBFUSCATION_UDP_TCP_PORT_AUTOMATIC",
                tableName: "VPNSettings",
                value: "Automatic",
                comment: ""
            )
        case .port80:
            "80"
        case .port5001:
            "5001"
        }
    }
}

public enum WireGuardObfuscationShadowsockPort: Codable, Equatable, CustomStringConvertible {
    case automatic
    case custom(UInt16)

    public var portValue: UInt16? {
        switch self {
        case .automatic:
            nil
        case let .custom(port):
            port
        }
    }

    public var description: String {
        switch self {
        case .automatic:
            NSLocalizedString(
                "WIREGUARD_OBFUSCATION_SHADOWSOCKS_PORT_AUTOMATIC",
                tableName: "VPNSettings",
                value: "Automatic",
                comment: ""
            )
        case let .custom(port):
            String(port)
        }
    }
}

// Can't deprecate the whole type since it'll yield a lint warning when decoding
// port in `WireGuardObfuscationSettings`.
private enum WireGuardObfuscationPort: UInt16, Codable {
    @available(*, deprecated, message: "Use `udpOverTcpPort` instead")
    case automatic = 0
    @available(*, deprecated, message: "Use `udpOverTcpPort` instead")
    case port80 = 80
    @available(*, deprecated, message: "Use `udpOverTcpPort` instead")
    case port5001 = 5001
}

public struct WireGuardObfuscationSettings: Codable, Equatable {
    @available(*, deprecated, message: "Use `udpOverTcpPort` instead")
    private var port: WireGuardObfuscationPort = .automatic

    public var state: WireGuardObfuscationState
    public var udpOverTcpPort: WireGuardObfuscationUdpOverTcpPort
    public var shadowsocksPort: WireGuardObfuscationShadowsockPort

    public init(
        state: WireGuardObfuscationState = .automatic,
        udpOverTcpPort: WireGuardObfuscationUdpOverTcpPort = .automatic,
        shadowsocksPort: WireGuardObfuscationShadowsockPort = .automatic
    ) {
        self.state = state
        self.udpOverTcpPort = udpOverTcpPort
        self.shadowsocksPort = shadowsocksPort
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        state = try container.decode(WireGuardObfuscationState.self, forKey: .state)
        shadowsocksPort = try container.decodeIfPresent(
            WireGuardObfuscationShadowsockPort.self,
            forKey: .shadowsocksPort
        ) ?? .automatic

        if let port = try? container.decodeIfPresent(WireGuardObfuscationUdpOverTcpPort.self, forKey: .udpOverTcpPort) {
            udpOverTcpPort = port
        } else if let port = try? container.decodeIfPresent(WireGuardObfuscationPort.self, forKey: .port) {
            switch port {
            case .automatic:
                udpOverTcpPort = .automatic
            case .port80:
                udpOverTcpPort = .port80
            case .port5001:
                udpOverTcpPort = .port5001
            }
        } else {
            udpOverTcpPort = .automatic
        }
    }
}
