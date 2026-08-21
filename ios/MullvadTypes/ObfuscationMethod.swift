// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

/// Describes the resolved obfuscation method with all required parameters.
public enum ObfuscationMethod: Equatable, Codable, Sendable {
    case off
    case udpOverTcp
    case shadowsocks
    case quic(hostname: String, token: String)
    case lwo

    public var isEnabled: Bool {
        switch self {
        case .off:
            false
        case .udpOverTcp, .shadowsocks, .quic, .lwo:
            true
        }
    }
}
