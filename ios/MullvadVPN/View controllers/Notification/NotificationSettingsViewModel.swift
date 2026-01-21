//
//  NotificationSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import SwiftUI
import UserNotifications

@MainActor
protocol NotificationSettingsViewModelProtocol: ObservableObject {
    var isNotificationsAllowed: Bool { get set }
    var settings: NotificationSettings { get set }

    func binding(for key: NotificationKeys) -> Binding<Bool>
    func all() -> Binding<Bool>
    func checkNotificationPermission()
    func openAppSettings()
}

final class NotificationSettingsViewModel: NotificationSettingsViewModelProtocol {
    @Published var isNotificationsAllowed: Bool = false
    @Published var settings: NotificationSettings = NotificationSettings()

    init(settings: NotificationSettings) {
        self.settings = settings
    }

    func checkNotificationPermission() {
        Task { @MainActor in
            self.isNotificationsAllowed = await UNUserNotificationCenter.isAllowed
        }
    }

    func openAppSettings() {
        if let url = URL(string: UIApplication.openNotificationSettingsURLString) {
            if UIApplication.shared.canOpenURL(url) {
                UIApplication.shared.open(url)
            }
        }
    }

    func binding(for key: NotificationKeys) -> Binding<Bool> {
        Binding(
            get: { self.settings[key] && self.isNotificationsAllowed },
            set: { self.settings[key] = $0 && self.isNotificationsAllowed }
        )
    }

    func all() -> Binding<Bool> {
        Binding(
            get: { self.settings.allAreEnabled && self.isNotificationsAllowed },
            set: { value in
                NotificationKeys.allCases.forEach { key in
                    self.settings[key] = value && self.isNotificationsAllowed
                }
            }
        )
    }
}
