//
//  AccountExpiry.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
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

    var formattedRemainingTime: String {
        let remainingTime = relativeFormatter.string(from: Date(), to: date)!
        let localizedString = NSLocalizedString("%@ left", tableName: "AccountExpiry", comment: "")

        return String(format: localizedString, remainingTime)
    }

    var formattedDate: String {
        return DateFormatter.localizedString(from: date, dateStyle: .medium, timeStyle: .medium)
    }

}
