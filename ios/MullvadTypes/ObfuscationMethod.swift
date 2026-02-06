//
//  ObfuscationMethod.swift
//  MullvadTypes
//
//  Created by Emīls on 2026-01-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Describes the resolved obfuscation method with all required parameters.
public enum ObfuscationMethod: Equatable, Codable, Sendable {
    case off
    case udpOverTcp
    case shadowsocks
    case quic(hostname: String, token: String)

    public var isEnabled: Bool {
        switch self {
        case .off:
            false
        case .udpOverTcp, .shadowsocks, .quic:
            true
        }
    }
}
