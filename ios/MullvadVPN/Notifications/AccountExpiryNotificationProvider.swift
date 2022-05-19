//
//  AccountExpiryNotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 03/06/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UserNotifications

let accountExpiryNotificationIdentifier = "net.mullvad.MullvadVPN.AccountExpiryNotification"
let accountExpiryDefaultTriggerInterval = 3

class AccountExpiryNotificationProvider: NotificationProvider, SystemNotificationProvider, InAppNotificationProvider, AccountObserver {
    private var accountExpiry: Date?

    /// Interval prior to expiry used to calculate when to trigger notifications.
    private let triggerInterval: Int

    override var identifier: String {
        return accountExpiryNotificationIdentifier
    }

    init(triggerInterval: Int = accountExpiryDefaultTriggerInterval) {
        self.triggerInterval = triggerInterval

        super.init()

        accountExpiry = Account.shared.expiry
        Account.shared.addObserver(self)
    }

    private var trigger: UNNotificationTrigger? {
        guard let accountExpiry = accountExpiry else { return nil }

        // Subtract 3 days from expiry date
        guard let triggerDate = Calendar.current.date(byAdding: .day, value: -triggerInterval, to: accountExpiry) else { return nil }

        // Do not produce notification if less than 3 days left till expiry
        guard triggerDate > Date() else { return nil }

        // Create date components for calendar trigger
        let dateComponents = Calendar.current.dateComponents([.second, .minute, .hour, .day, .month, .year], from: triggerDate)

        return UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)
    }

    private var shouldRemovePendingOrDeliveredRequests: Bool {
        return accountExpiry == nil
    }

    // MARK: - SystemNotificationProvider

    var notificationRequest: UNNotificationRequest? {
        guard let trigger = trigger else { return nil }

        _ = NSLocalizedString(
            "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_TITLE",
            comment: "Title for system account expiry notification, fired 3 days prior to account expiry."
        )
        _ = NSLocalizedString(
            "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_BODY",
            comment: "Message for system account expiry notification, fired 3 days prior to account expiry."
        )

        let content = UNMutableNotificationContent()
        content.title = NSString.localizedUserNotificationString(forKey: "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_TITLE", arguments: nil)
        content.body = NSString.localizedUserNotificationString(forKey: "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_BODY", arguments: nil)
        content.sound = UNNotificationSound.default

        return UNNotificationRequest(
            identifier: accountExpiryNotificationIdentifier,
            content: content,
            trigger: trigger
        )
    }

    var shouldRemovePendingRequests: Bool {
        // Remove pending notifications when account expiry is not set (user logged out)
        return shouldRemovePendingOrDeliveredRequests
    }

    var shouldRemoveDeliveredRequests: Bool {
        // Remove delivered notifications when account expiry is not set (user logged out)
        return shouldRemovePendingOrDeliveredRequests
    }

    // MARK: - InAppNotificationProvider

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard let accountExpiry = accountExpiry else { return nil }

        // Subtract 3 days from expiry date
        guard let triggerDate = Calendar.current.date(byAdding: .day, value: -triggerInterval, to: accountExpiry) else { return nil }

        // Only produce in-app notification within the last 3 days till expiry
        let now = Date()
        guard triggerDate < now && now < accountExpiry else { return nil }

        // Format the remaining duration
        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .full
        formatter.allowedUnits = [.minute, .hour, .day]
        formatter.maximumUnitCount = 1

        guard let duration = formatter.string(from: now, to: accountExpiry) else { return nil }

        return InAppNotificationDescriptor(
            identifier: self.identifier,
            style: .warning,
            title: NSLocalizedString(
                "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_TITLE",
                comment: "Title for in-app notification, displayed within the last 3 days until account expiry."
            ),
            body: String(
                format: NSLocalizedString(
                    "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_BODY",
                    comment: "Message for in-app notification, displayed within the last 3 days until account expiry."
                ), duration
            )
        )
    }

    func account(_ account: Account, didUpdateExpiry expiry: Date) {
        self.accountExpiry = expiry
        invalidate()
    }

    func account(_ account: Account, didLoginWithToken token: String, expiry: Date) {
        self.accountExpiry = expiry
        invalidate()
    }

    func accountDidLogout(_ account: Account) {
        self.accountExpiry = nil
        invalidate()
    }

}
