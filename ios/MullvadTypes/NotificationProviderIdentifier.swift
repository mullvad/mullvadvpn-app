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
    case accountExpirySystemNotification = "AccountExpiryNotification"
    case newAppVersionSystemNotification = "NewAppVersionSystemNotification"
    case newAppVersionInAppNotification = "NewAppVersionInAppNotification"
    case accountExpiryInAppNotification = "AccountExpiryInAppNotification"
    case registeredDeviceInAppNotification = "RegisteredDeviceInAppNotification"
    case tunnelStatusNotificationProvider = "TunnelStatusNotificationProvider"
    case latestChangesInAppNotificationProvider = "LatestChangesInAppNotificationProvider"
    case `default` = "default"

    public var domainIdentifier: String {
        "net.mullvad.MullvadVPN.\(rawValue)"
    }
}
