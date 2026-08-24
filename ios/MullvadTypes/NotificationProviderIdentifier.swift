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

public enum NotificationPriority: Int, Comparable {
    case low = 1
    case medium = 2
    case high = 3
    case critical = 4

    public static func < (lhs: NotificationPriority, rhs: NotificationPriority) -> Bool {
        lhs.rawValue < rhs.rawValue
    }
}

public enum NotificationProviderIdentifier: String {
    case accountExpiryInAppNotification = "AccountExpiryInAppNotification"
    case accountExpirySystemNotification = "AccountExpiryNotification"
    case invalidShadowsocksCipherInAppNotificationProvider = "InvalidShadowsocksCipherInAppNotificationProvider"
    case latestChangesInAppNotificationProvider = "LatestChangesInAppNotificationProvider"
    case settingsMigrationInAppNotificationProvider = "settingsMigrationInAppNotificationProvider"
    case newAppVersionInAppNotification = "NewAppVersionInAppNotification"
    case newAppVersionSystemNotification = "NewAppVersionSystemNotification"
    case registeredDeviceInAppNotification = "RegisteredDeviceInAppNotification"
    case tunnelStatusNotificationProvider = "TunnelStatusNotificationProvider"

    case `default` = "default"

    public var domainIdentifier: String {
        "net.mullvad.MullvadVPN.\(rawValue)"
    }
}
