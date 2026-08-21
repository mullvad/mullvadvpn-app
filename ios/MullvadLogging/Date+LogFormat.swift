// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

extension Date {
    public var logFormatted: String {
        let formatter = DateFormatter()

        formatter.dateFormat = "dd/MM/yyyy @ HH:mm:ss"
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(abbreviation: "UTC")

        return formatter.string(from: self)
    }

    public var safeLogFormatted: String {
        let formatter = DateFormatter()

        formatter.dateFormat = "dd/MM/yyyy"
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(abbreviation: "UTC")

        return formatter.string(from: self)
    }

    public var logFileFormatted: String {
        let formatter = DateFormatter()

        formatter.dateFormat = "dd-MM-yyyy'T'HH:mm:ss"
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(abbreviation: "UTC")

        return formatter.string(from: self)
    }
}
