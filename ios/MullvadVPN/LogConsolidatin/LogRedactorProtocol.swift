//
//  LogRedactorProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation

enum RedactionRules {
    case containerPaths([URL])
    case accountNumbers
    case ipv4
    case ipv6
    case customStrings([String])
}

protocol LogRedactorProtocol {
    func redact(
        _ input: String,
        using rules: [RedactionRules]
    ) -> String
}
