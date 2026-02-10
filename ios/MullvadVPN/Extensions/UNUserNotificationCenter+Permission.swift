//
//  UNUserNotificationCenter+Permission.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import UserNotifications

extension UNUserNotificationCenter {
    static var isAllowed: Bool {
        get async {
            let authorizationStatus = await UNUserNotificationCenter.authorizationStatus
            return authorizationStatus == .authorized
        }
    }

    static var isDisabled: Bool {
        get async {
            let authorizationStatus = await UNUserNotificationCenter.authorizationStatus
            return authorizationStatus == .denied
        }
    }

    static var authorizationStatus: UNAuthorizationStatus {
        get async {
            let settings = await UNUserNotificationCenter.current().notificationSettings()
            return settings.authorizationStatus
        }
    }
}
