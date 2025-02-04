//
//  InfoModalConfig.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct InfoModalConfig {
    public let header: String
    public let preamble: String
    public let body: [String]

    public init(header: String, preamble: String, body: [String]) {
        self.header = header
        self.preamble = preamble
        self.body = body
    }
}
