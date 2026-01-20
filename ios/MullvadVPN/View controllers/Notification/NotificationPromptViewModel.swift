import MullvadLogging
//
//  NotificationPromptViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI
import UserNotifications

@MainActor
protocol NotificationPromptViewModelProtocol: ObservableObject {
    var rows: [RowType] { get }
    var isNotificationsDisabled: Bool { get }
    var isNotificationsAllowed: Bool { get }
    func checkNotificationPermission()
    func requestNotificationPermission(completion: @MainActor @Sendable @escaping (Bool) -> Void)
}

enum RowType: Identifiable {
    case header(image: Image, text: LocalizedStringKey)
    case message(LocalizedStringKey)
    case action(
        text: LocalizedStringKey,
        style: MainButtonStyle.Style,
        action: () -> Void)
    case emptyView

    var id: UUID {
        UUID()
    }
}

@MainActor
final class NotificationPromptViewModel: NotificationPromptViewModelProtocol {

    @Published var isNotificationsDisabled: Bool = false
    @Published var isNotificationsAllowed: Bool = false
    private var logger = Logger(label: "NotificationManager")

    var rows: [RowType] {
        [
            .emptyView,
            .header(image: .mullvadIconAlert, text: "Set up notifications"),
            .message("Stay informed about your VPN connection and any actions needed to ensure it works correctly."),
            .message("We will never send you any ads or tips."),
            .emptyView,
            isNotificationsDisabled
                ? RowType.message(
                    "Notifications for Mullvad VPN are disabled on this device. Please change your system settings for Mullvad VPN if you wish to enable them again. These settings can be changed at any time."
                )
                : RowType.message("These settings can be changed at any time"),
            .action(
                text: "Enable notifications",
                style: .success,
                action: { [weak self] in
                    guard let self else { return }
                    if isNotificationsDisabled {
                        if let url = URL(string: UIApplication.openNotificationSettingsURLString) {
                            if UIApplication.shared.canOpenURL(url) {
                                UIApplication.shared.open(url)
                            }
                        }
                    } else {
                        requestNotificationPermission { isGranted in
                            self.isNotificationsAllowed = isGranted
                        }
                    }
                }),
            .action(
                text: "Skip",
                style: .default,
                action: { [weak self] in
                    self?.isNotificationsAllowed = false
                }),
        ]
    }

    init() {
        Task { @MainActor in
            self.isNotificationsDisabled = await UNUserNotificationCenter.authorizationStatus == .denied
        }
    }

    func checkNotificationPermission() {
        Task { @MainActor in
            self.isNotificationsAllowed = await UNUserNotificationCenter.isAllowed
        }
    }

    func requestNotificationPermission(completion: @MainActor @Sendable @escaping (Bool) -> Void) {
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

}
