//
//  InAppNotificationProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Protocol describing in-app notification provider.
protocol InAppNotificationProvider: NotificationProviderProtocol {
    /// In-app notification descriptor.
    var notificationDescriptor: InAppNotificationDescriptor? { get }
}
