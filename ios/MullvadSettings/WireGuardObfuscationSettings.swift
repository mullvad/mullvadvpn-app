//
//  WireGuardObfuscationSettings.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Whether obfuscation is enabled and which method is used
///
/// `.automatic` means an algorithm will decide whether to use it or not.
public enum WireGuardObfuscationState: Codable {
    @available(*, deprecated, renamed: "udpTcp")
    case on

    case automatic
    case udpTcp
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
        case .on, .udpTcp:
            self = .udpTcp
        case .shadowsocks:
            self = .shadowsocks
        case .off:
            self = .off
        }
    }
}

/// The port to select when using obfuscation
///
/// `.automatic` means an algorith will decide between using `.port80`, `port5001` or `.custom`.
public enum WireGuardObfuscationPort: Codable, Equatable {
    case automatic
    case port80
    case port5001
    case custom(UInt16)

    /// The `UInt16` representation of the port.
    public var portValue: UInt16 {
        switch self {
        case .automatic:
            0
        case .port80:
            80
        case .port5001:
            5001
        case .custom(let port):
            port
        }
    }

    public init?(rawValue: UInt16) {
        switch rawValue {
        case 0:
            self = .automatic
        case 80:
            self = .port80
        case 5001:
            self = .port5001
        default:
            self = .custom(rawValue)
        }
    }

    public init(from decoder: any Decoder) throws {
        if let container = try? decoder.container(keyedBy: CodingKeys.self) {
            var allKeys = ArraySlice(container.allKeys)
            guard let key = allKeys.popFirst(), allKeys.isEmpty else {
                throw DecodingError.typeMismatch(
                    WireGuardObfuscationPort.self,
                    DecodingError.Context.init(
                        codingPath: container.codingPath,
                        debugDescription: "Invalid number of keys found, expected one.",
                        underlyingError: nil
                    )
                )
            }

            switch key {
            case .automatic:
                self = .automatic
            case .port80:
                self = .port80
            case .port5001:
                self = .port5001
            case .custom:
                let nestedContainer = try container.nestedContainer(
                    keyedBy: WireGuardObfuscationPort.CustomCodingKeys.self, forKey: .custom
                )
                self = .custom(try nestedContainer.decode(
                    UInt16.self,
                    forKey: WireGuardObfuscationPort.CustomCodingKeys._0
                ))
            }
        } else {
            // Migration from old raw value.
            let value = try decoder.singleValueContainer().decode(UInt16.self)

            switch value {
            case 80:
                self = .port80
            case 5001:
                self = .port5001
            default:
                self = .automatic
            }
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
