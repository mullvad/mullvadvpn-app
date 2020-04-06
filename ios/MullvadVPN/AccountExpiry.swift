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

    private lazy var relativeFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .full
        formatter.allowedUnits = [.minute, .hour, .day, .month, .year]
        formatter.maximumUnitCount = 1

        return formatter
    }()

    init(date: Date) {
        self.date = date
    }

    var isExpired: Bool {
        return date <= Date()
    }

    var formattedRemainingTime: String? {
        return relativeFormatter.string(from: Date(), to: date)
    }

    var formattedDate: String {
        return DateFormatter.localizedString(from: date, dateStyle: .medium, timeStyle: .short)
    }

}
