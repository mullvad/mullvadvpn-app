//
//  NotificationSettings.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

enum NotificationKeys: String, CaseIterable {
    case account
    static var allCases: [NotificationKeys] {
        [.account]
    }

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

    init(isAccountNotificationEnabled: Bool = true) {
        self.isAccountNotificationEnabled = isAccountNotificationEnabled
    }

    subscript(key: NotificationKeys) -> Bool {
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
