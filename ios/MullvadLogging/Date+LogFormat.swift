//
//  Date+LogFormat.swift
//  LogFormatting
//
//  Created by pronebird on 09/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Date {
    public var logFormatted: String {
        logFileFormatted.replacingOccurrences(of: "T", with: " ")
    }

    public var logFileFormatted: String {
        let formatter = DateFormatter()

        formatter.dateFormat = "dd-MM-yyyy'T'HH:mm:ss"
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(abbreviation: "UTC")

        return formatter.string(from: self)
    }
}
