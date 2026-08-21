// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import Foundation
import MullvadSettings
import MullvadTypes
import UIKit

class LatestChangesNotificationProvider: NotificationProvider, InAppNotificationProvider, @unchecked Sendable {
    private var appPreferences: AppPreferencesDataSource
    private let appVersion: String = Bundle.main.productVersion

    init(appPreferences: AppPreferencesDataSource) {
        self.appPreferences = appPreferences
    }

    var shouldShowNotification: Bool {
        // If this is the first installation, no notification will be shown.
        guard !appPreferences.lastSeenChangeLogVersion.isEmpty else { return false }
        // Display the notification only if the app is updated from a previously installed version.
        return appPreferences.lastSeenChangeLogVersion != appVersion
    }

    override var identifier: NotificationProviderIdentifier {
        .latestChangesInAppNotificationProvider
    }

    override var priority: NotificationPriority {
        .low
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        defer {
            // Always update the last seen version
            appPreferences.lastSeenChangeLogVersion = appVersion
        }

        guard shouldShowNotification else { return nil }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .success,
            title: NSLocalizedString("NEW VERSION INSTALLED", comment: ""),
            body: createNotificationBody(),
            button: createCloseButtonAction(),
            tapAction: createTapAction()
        )
    }

    private func createNotificationBody() -> NSAttributedString {
        NSAttributedString(
            markdownString: NSLocalizedString("**Tap here** to see what’s new", comment: ""),
            options: MarkdownStylingOptions(
                font: .preferredFont(forTextStyle: .body)
            )
        ) { _, _ in
            [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
        }
    }

    private func createCloseButtonAction() -> InAppNotificationAction {
        InAppNotificationAction(
            image: UIImage.Buttons.closeSmall,
            handler: { [weak self] in
                self?.invalidate()
            }
        )
    }

    private func createTapAction() -> InAppNotificationAction {
        InAppNotificationAction(
            handler: { [weak self] in
                guard let self else { return }
                self.invalidate()
                NotificationManager.shared.notificationProvider(self, didReceiveAction: "\(self.identifier)")
            }
        )
    }
}
