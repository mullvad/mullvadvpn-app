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
import MullvadTypes

/// Base protocol for notification providers.
protocol NotificationProviderProtocol {
    /// Unique provider identifier used to identify notification providers and notifications
    /// produced by them.
    var identifier: NotificationProviderIdentifier { get }

    /// The priority level of the notification, used to determine the order in which notifications
    /// should be displayed. Higher priority notifications are displayed first.
    var priority: NotificationPriority { get }

    /// Tell notification manager to update the associated notification.
    func invalidate()
}
