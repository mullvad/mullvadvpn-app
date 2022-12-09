//
//  NotificationManagerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol NotificationManagerDelegate: AnyObject {
    func notificationManagerDidUpdateInAppNotifications(
        _ manager: NotificationManager,
        notifications: [InAppNotificationDescriptor]
    )
}
