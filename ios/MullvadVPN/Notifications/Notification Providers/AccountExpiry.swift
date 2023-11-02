//
//  AccountExpiry.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct AccountExpiry {
    var expiryDate: Date?

    var triggerDate: Date? {
        guard let expiryDate else { return nil }

        return Calendar.current.date(
            byAdding: .day,
            value: -NotificationConfiguration.closeToExpiryTriggerInterval,
            to: expiryDate
        )
    }

    var formattedDuration: String? {
        let now = Date()

        guard
            let expiryDate,
            let triggerDate,
            let duration = CustomDateComponentsFormatting.localizedString(
                from: now,
                to: expiryDate,
                unitsStyle: .full
            ),
            now >= triggerDate,
            now < expiryDate
        else {
            return nil
        }

        return duration
    }
}
