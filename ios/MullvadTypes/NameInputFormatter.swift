//
//  NameInputFormatter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-05-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public struct NameInputFormatter {
    public static let maxLength = 30

    public static func format(_ string: String, maxLength: Int = Self.maxLength) -> String {
        String(string.trimmingCharacters(in: .whitespaces).prefix(maxLength))
    }
}
