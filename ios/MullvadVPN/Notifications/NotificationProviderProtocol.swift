//
//  NotificationProviderProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Base protocol for notification providers.
protocol NotificationProviderProtocol {
    /// Unique provider identifier used to identify notification providers and notifications
    /// produced by them.
    var identifier: String { get }

    /// Tell notifcation manager to update the associated notification.
    func invalidate()
}
