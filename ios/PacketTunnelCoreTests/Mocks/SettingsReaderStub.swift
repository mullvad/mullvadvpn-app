// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes
import PacketTunnelCore

@testable import MullvadSettings

/// Settings reader stub that can be configured with a block to provide the desired behavior when testing.
struct SettingsReaderStub: SettingsReaderProtocol {
    let block: () throws -> Settings

    func read() throws -> Settings {
        return try block()
    }
}

extension SettingsReaderStub {
    /// Initialize non-fallible settings reader stub that will always return the same static configuration generated at the time of creation.
    static func staticConfiguration() -> SettingsReaderStub {
        let staticSettings = Settings(
            privateKey: WireGuard.PrivateKey(),
            interfaceAddresses: [IPAddressRange(from: "127.0.0.1/32")!],
            tunnelSettings: LatestTunnelSettings(
                relayConstraints: RelayConstraints(),
                dnsSettings: DNSSettings(),
                wireGuardObfuscation: WireGuardObfuscationSettings(state: .off),
                tunnelQuantumResistance: .on,
                tunnelMultihopState: .never,
                daita: DAITASettings()
            )
        )

        return SettingsReaderStub {
            return staticSettings
        }
    }

    static func noPostQuantumConfiguration() -> SettingsReaderStub {
        let staticSettings = Settings(
            privateKey: WireGuard.PrivateKey(),
            interfaceAddresses: [IPAddressRange(from: "127.0.0.1/32")!],
            tunnelSettings: LatestTunnelSettings(
                relayConstraints: RelayConstraints(),
                dnsSettings: DNSSettings(),
                wireGuardObfuscation: WireGuardObfuscationSettings(state: .off),
                tunnelQuantumResistance: .off,
                tunnelMultihopState: .never,
                daita: DAITASettings()
            )
        )
        return SettingsReaderStub {
            return staticSettings
        }
    }
}
