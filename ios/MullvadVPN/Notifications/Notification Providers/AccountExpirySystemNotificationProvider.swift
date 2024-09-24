//
//  AccountExpirySystemNotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 03/06/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import UserNotifications

final class AccountExpirySystemNotificationProvider: NotificationProvider, SystemNotificationProvider {
    private var accountExpiry = AccountExpiry()
    private var tunnelObserver: TunnelBlockObserver?
    private var accountHasExpired = false

    init(tunnelManager: TunnelManager) {
        super.init()

        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                self?.invalidate(deviceState: tunnelManager.deviceState)
            },
            didUpdateTunnelStatus: { [weak self] tunnelManager, _ in
                self?.checkAccountExpiry(
                    tunnelStatus: tunnelManager.tunnelStatus,
                    deviceState: tunnelManager.deviceState
                )
            },
            didUpdateDeviceState: { [weak self] _, deviceState, _ in
                if self?.accountHasExpired == false {
                    self?.invalidate(deviceState: deviceState)
                }
            }
        )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    override var identifier: NotificationProviderIdentifier {
        .accountExpirySystemNotification
    }

    // MARK: - SystemNotificationProvider

    var notificationRequest: UNNotificationRequest? {
        let trigger = accountHasExpired ? triggerExpiry : triggerCloseToExpiry

        guard let trigger, let formattedRemainingDurationBody else {
            return nil
        }

        let content = UNMutableNotificationContent()
        content.title = formattedRemainingDurationTitle
        content.body = formattedRemainingDurationBody
        content.sound = .default

        return UNNotificationRequest(
            identifier: identifier.domainIdentifier,
            content: content,
            trigger: trigger
        )
    }

    var shouldRemovePendingRequests: Bool {
        // Remove pending notifications when account expiry is not set (user logged out)
        shouldRemovePendingOrDeliveredRequests
    }

    var shouldRemoveDeliveredRequests: Bool {
        // Remove delivered notifications when account expiry is not set (user logged out)
        shouldRemovePendingOrDeliveredRequests
    }

    // MARK: - Private

    private var triggerCloseToExpiry: UNNotificationTrigger? {
        guard let triggerDate = accountExpiry.nextTriggerDate(for: .system) else { return nil }

        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year],
            from: triggerDate
        )

        return UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)
    }

    private var triggerExpiry: UNNotificationTrigger {
        // When scheduling a user notification we need to make sure that the date has not passed
        // when it's actually added to the system. Giving it a one second leeway lets us be sure
        // that this is the case.
        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year],
            from: Date().addingTimeInterval(1)
        )

        return UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)
    }

    private var shouldRemovePendingOrDeliveredRequests: Bool {
        return accountExpiry.expiryDate == nil
    }

    private func checkAccountExpiry(tunnelStatus: TunnelStatus, deviceState: DeviceState) {
        if !accountHasExpired {
            if case .accountExpired = tunnelStatus.observedState.blockedState?.reason {
                accountHasExpired = true
            }

            if accountHasExpired {
                invalidate(deviceState: deviceState)
            }
        }
    }

    private func invalidate(deviceState: DeviceState) {
        accountExpiry.expiryDate = deviceState.accountData?.expiry
        invalidate()
    }
}

extension AccountExpirySystemNotificationProvider {
    private var formattedRemainingDurationTitle: String {
        accountHasExpired
            ? NSLocalizedString(
                "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_TITLE",
                tableName: "AccountExpiry",
                value: "Account credit has expired",
                comment: "Title for system account expiry notification, fired on account expiry."
            )
            : NSLocalizedString(
                "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_TITLE",
                tableName: "AccountExpiry",
                value: "Account credit expires soon",
                comment: "Title for system account expiry notification, fired X days prior to account expiry."
            )
    }

    private var formattedRemainingDurationBody: String? {
        guard !accountHasExpired else { return expiredText }

        switch accountExpiry.daysRemaining(for: .system)?.day {
        case .none:
            return nil
        case 1:
            return singleDayText
        default:
            return multipleDaysText
        }
    }

    private var expiredText: String {
        NSLocalizedString(
            "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_BODY",
            tableName: "AccountExpiry",
            value: """
            Blocking internet: Your time on this account has expired. To continue using the internet, \
            please add more time or disconnect the VPN.
            """,
            comment: "Message for in-app notification, displayed on account expiry while connected to VPN."
        )
    }

    private var singleDayText: String {
        NSLocalizedString(
            "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_BODY",
            tableName: "AccountExpiry",
            value: "You have one day left on this account. Please add more time to continue using the VPN.",
            comment: "Message for in-app notification, displayed within the last 1 day until account expiry."
        )
    }

    private var multipleDaysText: String? {
        guard
            let expiryDate = accountExpiry.expiryDate,
            let nextTriggerDate = accountExpiry.nextTriggerDate(for: .system),
            let duration = CustomDateComponentsFormatting.localizedString(
                from: nextTriggerDate,
                to: expiryDate,
                unitsStyle: .full
            )
        else { return nil }

        return String(format: NSLocalizedString(
            "ACCOUNT_EXPIRY_SYSTEM_NOTIFICATION_BODY",
            tableName: "AccountExpiry",
            value: "You have %@ left on this account.",
            comment: "Message for in-app notification, displayed within the last X days until account expiry."
        ), duration.lowercased())
    }
}
