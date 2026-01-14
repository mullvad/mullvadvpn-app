//
//  LastVersionCheck.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public struct LastVersionCheck {
    public var version: String
    public var date: Date
    public var seenByUser: Bool

    public init(version: String, date: Date, seenByUser: Bool) {
        self.version = version
        self.date = date
        self.seenByUser = seenByUser
    }
}
