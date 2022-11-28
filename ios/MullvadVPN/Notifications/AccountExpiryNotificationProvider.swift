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
private let defaultTriggerInterval = 3

class AccountExpiryNotificationProvider: NotificationProvider, SystemNotificationProvider,
    InAppNotificationProvider, TunnelObserver
{
    private var accountExpiry: Date?

    /// Interval prior to expiry used to calculate when to trigger notifications.
    private let triggerInterval: Int

    override var identifier: String {
        return accountExpiryNotificationIdentifier
    }

    init(tunnelManager: TunnelManager, triggerInterval: Int = defaultTriggerInterval) {
        self.triggerInterval = triggerInterval

        super.init()

        tunnelManager.addObserver(self)
        accountExpiry = tunnelManager.deviceState.accountData?.expiry
    }

    private var trigger: UNNotificationTrigger? {
        guard let accountExpiry = accountExpiry else { return nil }

        // Subtract 3 days from expiry date
        guard let triggerDate = Calendar.current.date(
            byAdding: .day,
            value: -triggerInterval,
            to: accountExpiry
        ) else { return nil }

        // Do not produce notification if less than 3 days left till expiry
        guard triggerDate > Date() else { return nil }

        // Create date components for calendar trigger
        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year],
            from: triggerDate
        )

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
        content.title = NSString.localizedUserNotificationString(
            forKey: "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_TITLE",
            arguments: nil
        )
        content.body = NSString.localizedUserNotificationString(
            forKey: "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_BODY",
            arguments: nil
        )
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
        guard let triggerDate = Calendar.current.date(
            byAdding: .day,
            value: -triggerInterval,
            to: accountExpiry
        ) else { return nil }

        // Only produce in-app notification within the last 3 days till expiry
        let now = Date()
        guard triggerDate < now, now < accountExpiry else { return nil }

        // Format the remaining duration
        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .full
        formatter.allowedUnits = [.minute, .hour, .day]
        formatter.maximumUnitCount = 1

        guard let duration = formatter.string(from: now, to: accountExpiry) else { return nil }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString(
                "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_TITLE",
                value: "ACCOUNT CREDIT EXPIRES SOON",
                comment: "Title for in-app notification, displayed within the last 3 days until account expiry."
            ),
            body: String(
                format: NSLocalizedString(
                    "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_BODY",
                    value: "%@ left. Buy more credit.",
                    comment: "Message for in-app notification, displayed within the last 3 days until account expiry."
                ), duration
            )
        )
    }

    private func invalidate(deviceState: DeviceState) {
        accountExpiry = deviceState.accountData?.expiry
        invalidate()
    }

    // MARK: - TunnelObserver

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        invalidate(deviceState: manager.deviceState)
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        // no-op
    }

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        invalidate(deviceState: manager.deviceState)
    }
}
