//
//  NotificationProviderIdentifier.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-05-10.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum NotificationProviderIdentifier: String {
    case accountExpirySystemNotification = "AccountExpiryNotification"
    case accountExpiryInAppNotification = "AccountExpiryInAppNotification"
    case registeredDeviceInAppNotification = "RegisteredDeviceInAppNotification"
    case tunnelStatusNotificationProvider = "TunnelStatusNotificationProvider"
    case `default` = "default"

    var domainIdentifier: String {
        "net.mullvad.MullvadVPN.\(rawValue)"
    }
}
