//
//  AccountExpiryInAppNotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 12/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let triggerInterval = 3
private let refreshInterval = 60

final class AccountExpiryInAppNotificationProvider: NotificationProvider, InAppNotificationProvider
{
    private var accountExpiry: Date?
    private var tunnelObserver: TunnelBlockObserver?
    private var timer: DispatchSourceTimer?

    init(tunnelManager: TunnelManager) {
        super.init()

        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                self?.invalidate(deviceState: tunnelManager.deviceState)
            },
            didUpdateDeviceState: { [weak self] tunnelManager, deviceState in
                self?.invalidate(deviceState: deviceState)
            }
        )
        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
    }

    override var identifier: String {
        return "net.mullvad.MullvadVPN.AccountExpiryInAppNotification"
    }

    // MARK: - InAppNotificationProvider

    var notificationDescriptor: InAppNotificationDescriptor? {
        let now = Date()
        guard let accountExpiry = accountExpiry, let triggerDate = triggerDate, now >= triggerDate,
              now < accountExpiry
        else {
            return nil
        }

        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .full
        formatter.allowedUnits = [.minute, .hour, .day]
        formatter.maximumUnitCount = 1

        let duration: String?
        if accountExpiry.timeIntervalSince(now) < 60 {
            duration = NSLocalizedString(
                "ACCOUNT_EXPIRY_INAPP_NOTIFICATION_LESS_THAN_ONE_MINUTE",
                value: "Less than a minute",
                comment: ""
            )
        } else {
            duration = formatter.string(from: now, to: accountExpiry)
        }

        guard let duration = duration else { return nil }

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

    // MARK: - Private

    private var triggerDate: Date? {
        guard let accountExpiry = accountExpiry else { return nil }

        return Calendar.current.date(
            byAdding: .day,
            value: -triggerInterval,
            to: accountExpiry
        )
    }

    private func invalidate(deviceState: DeviceState) {
        updateExpiry(deviceState: deviceState)
        updateTimer()
        invalidate()
    }

    private func updateExpiry(deviceState: DeviceState) {
        accountExpiry = deviceState.accountData?.expiry
    }

    private func updateTimer() {
        timer?.cancel()

        guard let triggerDate = triggerDate else {
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
            repeating: .seconds(refreshInterval)
        )
        timer.activate()

        self.timer = timer
    }

    private func timerDidFire() {
        let shouldCancelTimer = accountExpiry.map { $0 <= Date() } ?? true

        if shouldCancelTimer {
            timer?.cancel()
        }

        invalidate()
    }
}
