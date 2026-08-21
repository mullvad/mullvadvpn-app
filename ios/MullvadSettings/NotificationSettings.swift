// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import MullvadTypes

public enum NotificationKeys: String, CaseIterable {
    case account

    var keyPath: KeyPath<NotificationSettings, Bool> {
        switch self {
        case .account:
            \.isAccountNotificationEnabled
        }
    }

    var writableKeyPath: WritableKeyPath<NotificationSettings, Bool> {
        switch self {
        case .account:
            \.isAccountNotificationEnabled
        }
    }
}

public struct NotificationSettings: Codable, Sendable, Equatable {
    public var isAccountNotificationEnabled: Bool

    public init(isAccountNotificationEnabled: Bool = true) {
        self.isAccountNotificationEnabled = isAccountNotificationEnabled
    }

    public subscript(key: NotificationKeys) -> Bool {
        get {
            self[keyPath: key.keyPath]
        }
        set {
            self[keyPath: key.writableKeyPath] = newValue
        }
    }

    public var allAreEnabled: Bool {
        NotificationKeys.allCases.allSatisfy { self[$0] }
    }
}
