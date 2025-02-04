//
//  InfoHeaderConfig.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct InfoHeaderConfig {
    public let body: String
    public let link: String

    public init(body: String, link: String) {
        self.body = body
        self.link = link
    }
}
