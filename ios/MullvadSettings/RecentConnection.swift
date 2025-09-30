//
//  RecentConnection.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

public struct RecentConnection: Codable, Sendable, Equatable {
    public let entry: UserSelectedRelays?
    public let exit: UserSelectedRelays
    var lastSelected: Date

    public init(entry: UserSelectedRelays? = nil, exit: UserSelectedRelays, lastSelected: Date = Date()) {
        self.entry = entry
        self.exit = exit
        self.lastSelected = Date()
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.entry == rhs.entry && lhs.exit == rhs.exit
    }
}
