//
//  AccountExpiry.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct AccountExpiry {
    enum Trigger {
        case system, inApp

        var dateIntervals: [Int] {
            switch self {
            case .system:
                NotificationConfiguration.closeToExpirySystemTriggerIntervals
            case .inApp:
                NotificationConfiguration.closeToExpiryInAppTriggerIntervals
            }
        }
    }

    private let calendar = Calendar.current

    var expiryDate: Date?

    func nextTriggerDate(for trigger: Trigger) -> Date? {
        let now = Date().secondsPrecision
        let triggerDates = triggerDates(for: trigger)

        // Get earliest trigger date and remove one day. Since we want to count whole days, If first
        // notification should trigger 3 days before account expiry, we need to start checking when
        // there's (less than) 4 days left.
        guard
            let expiryDate,
            let earliestDate = triggerDates.min(),
            let earliestTriggerDate = calendar.date(byAdding: .day, value: -1, to: earliestDate),
            now <= expiryDate.secondsPrecision,
            now > earliestTriggerDate.secondsPrecision
        else { return nil }

        let datesByTimeToTrigger = triggerDates.filter { date in
            now.secondsPrecision <= date.secondsPrecision // Ignore dates that have passed.
        }.sorted { date1, date2 in
            abs(date1.timeIntervalSince(now)) < abs(date2.timeIntervalSince(now))
        }

        return datesByTimeToTrigger.first
    }

    func daysRemaining(for trigger: Trigger) -> DateComponents? {
        let nextTriggerDate = nextTriggerDate(for: trigger)
        guard let expiryDate, let nextTriggerDate else { return nil }

        let dateComponents = calendar.dateComponents(
            [.day],
            from: Date().secondsPrecision,
            to: max(nextTriggerDate, expiryDate).secondsPrecision
        )

        return dateComponents
    }

    func triggerDates(for trigger: Trigger) -> [Date] {
        guard let expiryDate else { return [] }

        let dates = trigger.dateIntervals.compactMap {
            calendar.date(
                byAdding: .day,
                value: -$0,
                to: expiryDate
            )
        }

        return dates
    }
}

private extension Date {
    // Used to compare dates with a precision of a minimum of seconds.
    var secondsPrecision: Date {
        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year, .calendar],
            from: self
        )

        return dateComponents.date ?? self
    }
}
