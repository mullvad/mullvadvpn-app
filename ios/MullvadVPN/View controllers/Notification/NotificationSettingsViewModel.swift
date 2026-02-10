//
//  NotificationSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadSettings
import SwiftUI
import UserNotifications

@MainActor
protocol NotificationSettingsViewModelProtocol: ObservableObject {
    var isNotificationsAllowed: Bool { get set }
    var isNotificationsDisabled: Bool { get set }
    var settings: NotificationSettings { get set }

    func binding(for key: NotificationKeys) -> Binding<Bool>
    func checkNotificationPermission()
    func openAppSettings()
    func enableNotifications()
}

final class NotificationSettingsViewModel: NotificationSettingsViewModelProtocol {
    @Published var isNotificationsAllowed: Bool = false
    @Published var isNotificationsDisabled: Bool = false
    @Published var settings: NotificationSettings = NotificationSettings()
    private var logger = Logger(label: "NotificationSettingsViewModel")

    init(settings: NotificationSettings) {
        self.settings = settings
    }

    func checkNotificationPermission() {
        Task { @MainActor in
            self.isNotificationsAllowed = await UNUserNotificationCenter.isAllowed
            self.isNotificationsDisabled = await UNUserNotificationCenter.isDisabled
        }
    }

    func enableNotifications() {
        if isNotificationsDisabled {
            openAppSettings()
        } else {
            requestNotificationPermission { isGranted in
                self.isNotificationsAllowed = isGranted
            }
        }
    }

    private func requestNotificationPermission(completion: @MainActor @Sendable @escaping (Bool) -> Void) {
        let options: UNAuthorizationOptions = [.alert, .sound, .badge]

        UNUserNotificationCenter
            .current()
            .requestAuthorization(options: options) { granted, error in
                Task { @MainActor in
                    if let error = error as NSError? {
                        self.logger.error(
                            error: error,
                            message: "Failed to obtain user notifications authorizations"
                        )
                        completion(false)
                    } else {
                        completion(true)
                    }

                }
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
}
