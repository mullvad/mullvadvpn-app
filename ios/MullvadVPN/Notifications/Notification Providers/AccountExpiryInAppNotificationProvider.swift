//
//  AccountExpiryInAppNotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 12/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

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
                from: Date(),
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

final class AccountExpiryInAppNotificationProvider: NotificationProvider, InAppNotificationProvider {
    private var accountExpiry = AccountExpiry()
    private var tunnelObserver: TunnelBlockObserver?
    private var timer: DispatchSourceTimer?

    init(tunnelManager: TunnelManager) {
        super.init()

        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                self?.invalidate(deviceState: tunnelManager.deviceState)
            },
            didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.invalidate(deviceState: deviceState)
            }
        )
        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
    }

    override var identifier: NotificationProviderIdentifier {
        .accountExpiryInAppNotification
    }

    // MARK: - InAppNotificationProvider

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard let duration = accountExpiry.formattedDuration else {
            return nil
        }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString(
                "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_TITLE",
                value: "ACCOUNT CREDIT EXPIRES SOON",
                comment: "Title for in-app notification, displayed within the last 3 days until account expiry."
            ),
            body: .init(string: String(
                format: NSLocalizedString(
                    "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_BODY",
                    value: "%@ left. Buy more credit.",
                    comment: "Message for in-app notification, displayed within the last 3 days until account expiry."
                ), duration
            ))
        )
    }

    // MARK: - Private

    private func invalidate(deviceState: DeviceState) {
        updateExpiry(deviceState: deviceState)
        updateTimer()
        invalidate()
    }

    private func updateExpiry(deviceState: DeviceState) {
        accountExpiry.expiryDate = deviceState.accountData?.expiry
    }

    private func updateTimer() {
        timer?.cancel()

        guard let triggerDate = accountExpiry.triggerDate else {
            return
        }

        let now = Date()
        let fireDate = max(now, triggerDate)

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            self?.timerDidFire()
        }
        timer.schedule(
            wallDeadline: .now() + fireDate.timeIntervalSince(now),
            repeating: .seconds(NotificationConfiguration.closeToExpiryInAppNotificationRefreshInterval)
        )
        timer.activate()

        self.timer = timer
    }

    private func timerDidFire() {
        let shouldCancelTimer = accountExpiry.expiryDate.map { $0 <= Date() } ?? true

        if shouldCancelTimer {
            timer?.cancel()
        }

        invalidate()
    }
}
