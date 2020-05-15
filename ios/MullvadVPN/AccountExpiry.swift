//
//  AccountExpiry.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

class AccountExpiry {
    let date: Date

    init(date: Date) {
        self.date = date
    }

    var isExpired: Bool {
        return date <= Date()
    }

    var formattedRemainingTime: String? {
        return CustomDateComponentsFormatting.localizedString(
            from: Date(),
            to: date,
            unitsStyle: .full
        )
    }

    var formattedDate: String {
        return DateFormatter.localizedString(from: date, dateStyle: .medium, timeStyle: .short)
    }

}
