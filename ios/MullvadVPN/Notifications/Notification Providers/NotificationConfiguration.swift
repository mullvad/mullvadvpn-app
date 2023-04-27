//
//  NotificationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 27/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum NotificationConfiguration {
    /**
     Duration measured in days, before the account expiry, when notification is scheduled to remind user to add more
     time on account.
     */
    static let closeToExpiryTriggerInterval = 3

    /**
     Time interval measured in seconds at which to refresh account expiry in-app notification, which reformats
     the duration until account expiry over time.
     */
    static let closeToExpiryInAppNotificationRefreshInterval = 60
}
