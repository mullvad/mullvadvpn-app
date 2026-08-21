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

enum NotificationConfiguration {
    /**
     Duration measured in days, before the account expiry, when a system notification is scheduled to remind user
     to add more time on account.
     */
    static let closeToExpirySystemTriggerIntervals = [3, 1]

    /**
     Duration measured in days, before the account expiry, when an in-app notification is scheduled to remind user
     to add more time on account.
     */
    static let closeToExpiryInAppTriggerIntervals: [Int] = [3, 2, 1, 0]

    /**
     Time interval measured in seconds at which to refresh account expiry in-app notification, which reformats
     the duration until account expiry over time.
     */
    static let closeToExpiryInAppNotificationRefreshInterval = 60
}
