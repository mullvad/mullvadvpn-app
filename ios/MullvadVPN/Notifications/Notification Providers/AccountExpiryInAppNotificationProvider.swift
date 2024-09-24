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
            didUpdateTunnelStatus: { [weak self] tunnelManager, _ in
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
        guard let durationText = remainingDaysText else {
            return nil
        }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: durationText,
            body: NSAttributedString(string: NSLocalizedString(
                "ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_BODY",
                value: "You can add more time via the account view or website to continue using the VPN.",
                comment: "Title for in-app notification, displayed within the last X days until account expiry."
            ))
        )
    }

    // MARK: - Private

    private func invalidate(deviceState: DeviceState) {
        accountExpiry.expiryDate = deviceState.accountData?.expiry
        updateTimer()
        invalidate()
    }

    private func updateTimer() {
        timer?.cancel()

        guard let triggerDate = accountExpiry.nextTriggerDate(for: .inApp) else {
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

extension AccountExpiryInAppNotificationProvider {
    private var remainingDaysText: String? {
        guard
            let expiryDate = accountExpiry.expiryDate,
            let nextTriggerDate = accountExpiry.nextTriggerDate(for: .inApp),
            let duration = CustomDateComponentsFormatting.localizedString(
                from: nextTriggerDate,
                to: expiryDate,
                unitsStyle: .full
            )
        else { return nil }

        return String(format: NSLocalizedString(
            "ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_TITLE",
            tableName: "AccountExpiry",
            value: "%@ left on this account",
            comment: "Message for in-app notification, displayed within the last X days until account expiry."
        ), duration).uppercased()
    }
}
