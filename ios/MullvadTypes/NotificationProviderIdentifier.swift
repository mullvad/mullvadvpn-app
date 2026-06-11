//
//  NotificationProviderIdentifier.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-05-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
