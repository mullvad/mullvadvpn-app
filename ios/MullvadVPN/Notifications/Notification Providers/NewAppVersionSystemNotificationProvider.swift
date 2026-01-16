//
//  NewAppVersionSystemNotificationProvider.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UserNotifications

final class NewAppVersionSystemNotificationProvider: NotificationProvider, SystemNotificationProvider,
    @unchecked Sendable
{
    private let tunnelManager: TunnelManager
    private let appStoreMetaDataService: AppStoreMetaDataService

    private let checkInterval: TimeInterval = 86_400  // 24 hours
    private var timer: DispatchSourceTimer? = DispatchSource.makeTimerSource()
    private var shouldSendNotification = false

    init(
        tunnelManager: TunnelManager,
        appStoreMetaDataService: AppStoreMetaDataService
    ) {
        self.tunnelManager = tunnelManager
        self.appStoreMetaDataService = appStoreMetaDataService

        super.init()

        scheduleTimer()
    }

    deinit {
        timer?.cancel()
    }

    override var identifier: NotificationProviderIdentifier {
        .newAppVersionSystemNotification
    }

    override var priority: NotificationPriority {
        .critical
    }

    // MARK: - SystemNotificationProvider

    var notificationRequest: UNNotificationRequest? {
        guard shouldSendNotification else { return nil }

        let content = UNMutableNotificationContent()
        content.title = NSLocalizedString("Update available", comment: "")
        content.body = NSLocalizedString(
            "Disable “Force all apps” or disconnect before updating in order not to lose network connectivity.",
            comment: ""
        )

        return UNNotificationRequest(identifier: identifier.domainIdentifier, content: content, trigger: trigger)
    }

    var shouldRemovePendingRequests: Bool {
        true
    }

    var shouldRemoveDeliveredRequests: Bool {
        false
    }

    // MARK: - Private

    private var trigger: UNNotificationTrigger {
        // When scheduling a user notification we need to make sure that the date has not passed
        // when it's actually added to the system. Giving it a few seconds leeway lets us be sure
        // that this is the case.
        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year],
            from: Date(timeIntervalSinceNow: 5)
        )

        return UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)
    }

    private func scheduleTimer() {
        timer?.setEventHandler {
            Task { [weak self] in
                guard let self else { return }

                if tunnelManager.tunnelStatus.state.isSecured, tunnelManager.settings.includeAllNetworks {
                    shouldSendNotification = (try? await appStoreMetaDataService.performVersionCheck()) ?? false
                    invalidate()
                }
            }
        }

        // Resume deadline if there's time left from previous check. Otherwise, fire away.
        let deadline = max(
            checkInterval - appStoreMetaDataService.appPreferences.lastVersionCheckDate.timeIntervalSinceNow, 0)

        timer?.schedule(deadline: .now() + deadline, repeating: .seconds(Int(checkInterval)))
        timer?.activate()
    }
}
