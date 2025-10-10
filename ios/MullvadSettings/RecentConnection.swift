//
//  RecentConnection.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-22.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

public struct RecentConnection: Codable, Sendable, Equatable {
    let entry: UserSelectedRelays?
    let exit: UserSelectedRelays

    public init(entry: UserSelectedRelays? = nil, exit: UserSelectedRelays) {
        self.entry = entry
        self.exit = exit
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.entry == rhs.entry && lhs.exit == rhs.exit
    }
}
