//
//  DateFormatter+Custom.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension DateFormatter {
    static func localizedString(from: Date, format: String) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = format
        return formatter.string(from: from)
    }
}
