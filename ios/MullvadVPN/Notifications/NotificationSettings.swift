//
//  NotificationSettings.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

enum NotificationKeys: String, CaseIterable {
    case account
    case connectionStatus
    static var allCases: [NotificationKeys] {
        [.account, .connectionStatus]
    }

    var keyPath: KeyPath<NotificationSettings, Bool> {
        switch self {
        case .account:
            \.isAccountNotificationEnabled
        case .connectionStatus:
            \.isConnectionStatusNotificationEnabled
        }
    }

    var writableKeyPath: WritableKeyPath<NotificationSettings, Bool> {
        switch self {
        case .account:
            \.isAccountNotificationEnabled
        case .connectionStatus:
            \.isConnectionStatusNotificationEnabled
        }
    }
}

struct NotificationSettings: Codable {
    var isAccountNotificationEnabled: Bool
    var isConnectionStatusNotificationEnabled: Bool

    init(
        isAccountNotificationEnabled: Bool = true,
        isConnectionStatusNotificationEnabled: Bool = true
    ) {
        self.isAccountNotificationEnabled = isAccountNotificationEnabled
        self.isConnectionStatusNotificationEnabled = isConnectionStatusNotificationEnabled
    }

    subscript(key: NotificationKeys) -> Bool {
        get {
            self[keyPath: key.keyPath]
        }
        set {
            self[keyPath: key.writableKeyPath] = newValue
        }
    }

    var allAreEnabled: Bool {
        NotificationKeys.allCases.allSatisfy { self[$0] }
    }
}
