// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import UserNotifications

final class NewAppVersionSystemNotificationHandler {
    private let appVersionService: AppVersionService
    private let settingsUpdater: SettingsUpdater
    private var tunnelSettings: LatestTunnelSettings
    private var observer: SettingsObserverBlock!

    init(
        appVersionService: AppVersionService,
        settingsUpdater: SettingsUpdater,
        tunnelSettings: LatestTunnelSettings
    ) {
        self.settingsUpdater = settingsUpdater
        self.appVersionService = appVersionService
        self.tunnelSettings = tunnelSettings

        self.observer = SettingsObserverBlock(
            didUpdateSettings: { [weak self] latestTunnelSettings in
                self?.tunnelSettings = latestTunnelSettings
            }
        )
        self.settingsUpdater.addObserver(observer)

        self.appVersionService.onNewAppVersion = { [weak self] in
            if tunnelSettings.includeAllNetworks.includeAllNetworksIsEnabled {
                self?.sendSystemNotification()
            }
        }

        appVersionService.scheduleTimer(deadline: .nextCheck)
    }

    private func sendSystemNotification() {
        let content = UNMutableNotificationContent()
        content.title = NSLocalizedString("Update available", comment: "")
        content.body = String(
            format: NSLocalizedString(
                "Disable “%@” or disconnect before updating in order not to lose network connectivity.",
                comment: ""
            ),
            NSLocalizedString("Force all apps", comment: "")
        )

        // When scheduling a user notification we need to make sure that the date has not passed
        // when it's actually added to the system. Giving it a few seconds leeway lets us be sure
        // that this is the case.
        let dateComponents = Calendar.current.dateComponents(
            [.second, .minute, .hour, .day, .month, .year],
            from: Date(timeIntervalSinceNow: 5)
        )
        let trigger = UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)

        let request = UNNotificationRequest(
            identifier: NotificationProviderIdentifier.newAppVersionSystemNotification.domainIdentifier,
            content: content,
            trigger: trigger
        )

        let identifier = request.identifier
        UNUserNotificationCenter.current().add(request) { error in
            if let error {
                Logger(label: "NewAppVersionSystemNoticationHandler").error(
                    "Failed to add notification request with identifier \(identifier). Error: \(error.description)"
                )
            }
        }
    }
}
