// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import UIKit

final class InvalidShadowsocksCipherInAppNotificationProvider: NotificationProvider, InAppNotificationProvider,
    @unchecked Sendable
{
    private var breadcrumbsObserver: BreadcrumbsObserver?
    private var shouldShowNotification = false

    override var identifier: NotificationProviderIdentifier {
        .invalidShadowsocksCipherInAppNotificationProvider
    }

    override var priority: NotificationPriority {
        .high
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard shouldShowNotification else {
            return nil
        }

        let string = NSMutableAttributedString(
            string: NSLocalizedString(
                "Please update it or enable a different one to be able to reach the API.", comment: "")
        )
        string.append(NSAttributedString(string: "\n"))
        string.append(
            NSAttributedString(
                string: NSLocalizedString("Tap to view API access methods", comment: ""),
                attributes: [
                    .font: UIFont.mullvadTinySemiBold,
                    .foregroundColor: UIColor.white,
                ]
            )
        )

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString("CUSTOM API ACCESS METHOD IS INVALID", comment: ""),
            body: string,
            tapAction: createTapAction(),
        )
    }

    init(breadcrumbsProvider: BreadcrumbsProvider) {
        super.init()

        let breadcrumbsObserver = BreadcrumbsBlockObserver { [weak self] in
            self?.shouldShowNotification = $0.contains(.warning(.apiAccess))
            self?.invalidate()
        }
        self.breadcrumbsObserver = breadcrumbsObserver
        breadcrumbsProvider.add(observer: breadcrumbsObserver)
    }

    // MARK: - Private

    private func createTapAction() -> InAppNotificationAction {
        InAppNotificationAction(
            handler: { [weak self] in
                guard let self else { return }
                NotificationManager.shared.notificationProvider(self, didReceiveAction: "\(identifier)")
            }
        )
    }
}
