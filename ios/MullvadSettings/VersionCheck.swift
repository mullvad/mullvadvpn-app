//
//  VersionCheck.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct VersionCheck: Codable {
    public var version: String
    public var date: Date
}
