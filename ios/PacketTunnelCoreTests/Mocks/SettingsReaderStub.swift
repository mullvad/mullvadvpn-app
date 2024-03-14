//
//  SettingsReaderStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import WireGuardKitTypes

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
            privateKey: PrivateKey(),
            interfaceAddresses: [IPAddressRange(from: "127.0.0.1/32")!],
            relayConstraints: RelayConstraints(),
            dnsServers: .gateway,
            obfuscation: WireGuardObfuscationSettings(state: .off, port: .automatic),
            tunnelQuantumResistance: .automatic
        )

        return SettingsReaderStub {
            return staticSettings
        }
    }
}
