//
//  RelayCandidates.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct RelayCandidates {
    public let entryRelays: [RelayWithLocation<REST.ServerRelay>]?
    public let exitRelays: [RelayWithLocation<REST.ServerRelay>]
    public init(
        entryRelays: [RelayWithLocation<REST.ServerRelay>]?,
        exitRelays: [RelayWithLocation<REST.ServerRelay>]
    ) {
        self.entryRelays = entryRelays
        self.exitRelays = exitRelays
    }
}
