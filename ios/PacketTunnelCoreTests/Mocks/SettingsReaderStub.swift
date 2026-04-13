//
//  SettingsReaderStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
