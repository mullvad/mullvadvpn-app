//
//  ObfuscationMethod.swift
//  MullvadTypes
//
//  Created by Emīls on 2026-01-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Describes the resolved obfuscation method with all required parameters.
public enum ObfuscationMethod: Equatable, Codable, Sendable {
    case off
    case udpOverTcp
    case shadowsocks
    case quic(hostname: String, token: String)
    case lwo
}
