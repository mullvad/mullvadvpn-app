//
//  LatestChangesNotificationProvider.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-01-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import UIKit

class LatestChangesNotificationProvider: NotificationProvider, InAppNotificationProvider, @unchecked Sendable {
    private var appPreferences: AppPreferencesDataSource
    private let appVersion: String = Bundle.main.productVersion

    init(appPreferences: AppPreferencesDataSource) {
        self.appPreferences = appPreferences
    }

    var isUpdated: Bool {
        guard !appPreferences.lastSeenChangeLogVersion.isEmpty else { return false }
        return appPreferences.lastSeenChangeLogVersion != appVersion
    }

    override var identifier: NotificationProviderIdentifier {
        .latestChangesInAppNotificationProvider
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        defer {
            // Always update the last seen version
            appPreferences.lastSeenChangeLogVersion = appVersion
        }

        guard isUpdated else { return nil }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .success,
            title: NSLocalizedString(
                "Latest_Changes_IN_APP_NOTIFICATION_TITLE",
                value: "NEW VERSION INSTALLED",
                comment: ""
            ),
            body: createNotificationBody(),
            button: createCloseButtonAction(),
            tapAction: createTapAction()
        )
    }

    private func createNotificationBody() -> NSAttributedString {
        NSAttributedString(
            markdownString: NSLocalizedString(
                "Latest_Changes_IN_APP_NOTIFICATION_BODY",
                value: "**Tap here** to see what’s new.",
                comment: ""
            ),
            options: MarkdownStylingOptions(font: UIFont.preferredFont(forTextStyle: .body)),
            applyEffect: { markdownType, _ in
                guard case .bold = markdownType else { return [:] }
                return [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
            }
        )
    }

    private func createCloseButtonAction() -> InAppNotificationAction {
        InAppNotificationAction(
            image: UIImage(resource: .iconCloseSml),
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
