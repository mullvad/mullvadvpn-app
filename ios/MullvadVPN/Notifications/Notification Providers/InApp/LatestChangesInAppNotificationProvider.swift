//
//  LastestChangesNotificationProvider.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-01-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import UIKit

class LatestChangesInAppNotificationProvider: NotificationProvider, InAppNotificationProvider, @unchecked Sendable {
    private var appPreferences: AppPreferencesDataSource
    private let appVersion = Bundle.main.productVersion

    init(appPreferences: AppPreferencesDataSource) {
        self.appPreferences = appPreferences
    }

    var isFirstLaunch: Bool {
        appPreferences.isAgreedToTermsOfService &&
            !appPreferences.lastSeenChangeLogVersion.isEmpty &&
            appPreferences.lastSeenChangeLogVersion != appVersion
    }

    override var identifier: NotificationProviderIdentifier {
        .latestChangesInAppNotificationProvider
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard isFirstLaunch else { return nil }
        return InAppNotificationDescriptor(identifier: identifier, style: .success, title: NSLocalizedString(
            "Latest_Changes_IN_APP_NOTIFICATION_TITLE",
            value: "NEW VERSION INSTALLED",
            comment: ""
        ), body: NSAttributedString(
            markdownString: NSLocalizedString(
                "Latest_Changes_IN_APP_NOTIFICATION_BODY",
                value: "**Tap here** to see what’s new.",
                comment: ""
            ),
            options: MarkdownStylingOptions(font: UIFont.preferredFont(forTextStyle: .body)),
            applyEffect: { markdownType, _ in
                guard case .bold = markdownType else {
                    return [:]
                }
                return [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
            }
        ), button: InAppNotificationAction(image: UIImage(resource: .iconCloseSml), handler: { [weak self] in
            guard let self else { return }
            appPreferences.lastSeenChangeLogVersion = appVersion
            invalidate()
        }), tapAction: InAppNotificationAction(handler: { [weak self] in
            guard let self else { return }
            appPreferences.lastSeenChangeLogVersion = appVersion
            invalidate()
            NotificationManager.shared.notificationProvider(self, didReceiveAction: "\(identifier)")
        }))
    }
}
