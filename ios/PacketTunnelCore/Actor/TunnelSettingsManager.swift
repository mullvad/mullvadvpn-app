//
//  TunnelSettingsManager.swift
//  PacketTunnelCore
//
//  Created by Mojgan on 2024-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct TunnelSettingsManager: SettingsReaderProtocol {
    let settingsReader: SettingsReaderProtocol
    let onLoadSettingsHandler: ((Settings) -> Void)?

    public init(settingsReader: SettingsReaderProtocol, onLoadSettingsHandler: ((Settings) -> Void)? = nil) {
        self.settingsReader = settingsReader
        self.onLoadSettingsHandler = onLoadSettingsHandler
    }

    public func read() throws -> Settings {
        let settings = try settingsReader.read()
        onLoadSettingsHandler?(settings)
        return settings
    }
}
