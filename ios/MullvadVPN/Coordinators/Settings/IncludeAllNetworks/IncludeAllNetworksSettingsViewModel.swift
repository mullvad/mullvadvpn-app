//
//  IncludeAllNetworksSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import SwiftUI
import UIKit
import UserNotifications

@MainActor
protocol IncludeAllNetworksSettingsViewModel: ObservableObject {
    var includeAllNetworksState: InclueAllNetworksState { get set }
    var localNetworkSharingState: LocalNetworkSharingState { get set }
    var consent: Bool { get set }

    var showEnableNotificationsAlert: Bool { get set }
    var showReconsiderNotificationsAlert: Bool { get set }
    var tunnelIsSecured: Bool { get }
}

class IncludeAllNetworksSettingsViewModelImpl: IncludeAllNetworksSettingsViewModel {
    enum Feature: String {
        case includeAllNetworks = "Force all apps"
        case localNetworkSharing = "Local network sharing"
    }

    var settings: IncludeAllNetworksSettings

    @Published var includeAllNetworksState: InclueAllNetworksState {
        didSet {
            if includeAllNetworksState == .on {
                checkNotificationPermissions { [weak self] status in
                    switch status {
                    case .notDetermined, .provisional:
                        self?.showEnableNotificationsAlert = true
                    case .denied, .ephemeral:
                        self?.showReconsiderNotificationsAlert = true
                    default:
                        break
                    }
                }
            }

            settings.includeAllNetworksState = includeAllNetworksState
            tunnelManager.updateSettings([.includeAllNetworks(settings)])
        }
    }

    @Published var localNetworkSharingState: LocalNetworkSharingState {
        didSet {
            settings.localNetworkSharingState = localNetworkSharingState
            tunnelManager.updateSettings([.includeAllNetworks(settings)])
        }
    }

    @Published var consent: Bool {
        didSet {
            appPreferences.includeAllNetworksConsent = consent
        }
    }

    @Published var showEnableNotificationsAlert: Bool = false
    @Published var showReconsiderNotificationsAlert: Bool = false

    var tunnelIsSecured: Bool {
        // Tunnel is considered secured if network is down and the tunnel state
        // is "secured".
        tunnelManager.tunnelStatus.state != .error(.offline)
            && tunnelManager.tunnelStatus.state.isSecured
    }

    let tunnelManager: TunnelManager
    var appPreferences: AppPreferencesDataSource

    init(tunnelManager: TunnelManager, appPreferences: AppPreferencesDataSource) {
        self.tunnelManager = tunnelManager
        self.appPreferences = appPreferences

        settings = IncludeAllNetworksSettings(
            includeAllNetworksState: tunnelManager.settings.includeAllNetworks.includeAllNetworksState,
            localNetworkSharingState: tunnelManager.settings.includeAllNetworks.localNetworkSharingState
        )

        includeAllNetworksState = settings.includeAllNetworksState
        localNetworkSharingState = settings.localNetworkSharingState
        consent = appPreferences.includeAllNetworksConsent
    }
}

// MARK: Notifications

extension IncludeAllNetworksSettingsViewModel {
    func navigateToAppSystemSettings() {
        if let appSettings = URL(string: UIApplication.openSettingsURLString),
            UIApplication.shared.canOpenURL(appSettings)
        {
            UIApplication.shared.open(appSettings)
        }
    }

    func requestNotificationPermissions(completion: ((Bool, Error?) -> Void)?) {
        let authorizationOptions: UNAuthorizationOptions = [.alert, .sound]
        nonisolated(unsafe) let completion = completion

        UNUserNotificationCenter.current().requestAuthorization(options: authorizationOptions) { granted, error in
            DispatchQueue.main.async {
                completion?(granted, error)
            }
        }
    }

    func checkNotificationPermissions(completion: @escaping (UNAuthorizationStatus) -> Void) {
        nonisolated(unsafe) let completion = completion

        UNUserNotificationCenter.current().getNotificationSettings { notificationSettings in
            let status = notificationSettings.authorizationStatus
            DispatchQueue.main.async {
                completion(status)
            }
        }
    }
}

// MARK: Alerts

extension IncludeAllNetworksSettingsViewModel {
    func getEnableNotificationsAlert(completion: @escaping () -> Void) -> MullvadAlert {
        let message =
            "We can send you a notification when an update is available so that you can disable "
            + "this feature or disconnect before updating. This can be changed at any time "
            + "in settings."

        return MullvadAlert(
            type: .warning,
            messages: [LocalizedStringKey(message)],
            actions: [
                MullvadAlert.Action(
                    type: .default,
                    title: "Enable notifications",
                    handler: { [weak self] in
                        self?.requestNotificationPermissions(completion: nil)
                        self?.showEnableNotificationsAlert = false
                        completion()
                    }
                ),
                MullvadAlert.Action(
                    type: .default,
                    title: "Got it!",
                    identifier: .includeAllNetworksNotificationsAlertDismissButton,
                    handler: { [weak self] in
                        self?.showEnableNotificationsAlert = false
                        completion()
                    }
                ),
            ]
        )
    }

    func getReconsiderNotificationsAlert(completion: @escaping () -> Void) -> MullvadAlert {
        let message = [
            ("You currently have notifications disabled. This means that we cannot send you a "
                + "notification when an update is available so that you can disable this "
                + "feature or disconnect before updating."),
            ("Please enable notifications to ensure that you do not lose network connectivity. "
                + "Would you like to continue anyways?"),
        ].joinedParagraphs()

        return MullvadAlert(
            type: .warning,
            messages: [LocalizedStringKey(message)],
            actions: [
                MullvadAlert.Action(
                    type: .default,
                    title: "Open system settings",
                    handler: { [weak self] in
                        self?.navigateToAppSystemSettings()
                        self?.showReconsiderNotificationsAlert = false
                        completion()
                    }
                ),
                MullvadAlert.Action(
                    type: .danger,
                    title: "Yes, continue",
                    identifier: .includeAllNetworksNotificationsAlertDismissButton,
                    handler: { [weak self] in
                        self?.showReconsiderNotificationsAlert = false
                        completion()
                    }
                ),
            ]
        )
    }

    func getLanSharingInfoAlert(completion: @escaping () -> Void) -> MullvadAlert {
        let messageInfo = [
            ("This feature allows access to other devices on the local network, "
                + "such as for sharing, printing, streaming, etc."),
            ("It does this by allowing network communication outside the tunnel "
                + "to local multicast and broadcast ranges as well as to and from "
                + "these private IP ranges:"),
        ].joinedParagraphs()

        let ipList = [
            " ∙ 10.0.0.0/8",
            " ∙ 172.16.0.0/12",
            " ∙ 192.168.0.0/16",
            " ∙ 169.254.0.0/16",
            " ∙ fe80::/10",
            " ∙ fc00::/7",
        ].joinedParagraphs(lineBreaks: 1)

        let messageAttention =
            "Attention: toggling “Local network sharing” requires restarting the VPN "
            + "connection."

        return MullvadAlert(
            type: .info,
            messages: [
                LocalizedStringKey(messageInfo),
                LocalizedStringKey(ipList),
                LocalizedStringKey(""),
                LocalizedStringKey(messageAttention),
            ],
            actions: [
                .init(
                    type: .default,
                    title: "Got it!",
                    identifier: .includeAllNetworksNotificationsAlertDismissButton,
                    handler: {
                        completion()
                    }
                )
            ]
        )
    }

    func getEnableFeatureAlert(
        feature: IncludeAllNetworksSettingsViewModelImpl.Feature,
        enabled: Bool,
        completion: @escaping () -> Void
    ) -> MullvadAlert? {
        let setValue: (Bool) -> Void = { [weak self] enabled in
            switch feature {
            case .includeAllNetworks:
                self?.includeAllNetworksState.isEnabled = enabled
            case .localNetworkSharing:
                self?.localNetworkSharingState.isEnabled = enabled
            }
        }

        guard tunnelIsSecured else {
            setValue(enabled)
            return nil
        }

        var message = [
            String(
                format:
                    "%@ “%@“ requires restarting the VPN connection, which will disconnect "
                    + "you and briefly expose your traffic. To prevent this, manually enable "
                    + "Airplane Mode and turn off Wi-Fi before continuing.",
                enabled ? "Enabling" : "Disabling",
                feature.rawValue
            ),
            String(
                format: "Would you like to continue to %@ “%@”?",
                enabled ? "enable" : "disable",
                feature.rawValue
            ),
        ]

        if !enabled && feature == .includeAllNetworks {
            message.insert("This will also disable “Local Network Sharing“.", at: 1)
        }

        return MullvadAlert(
            type: .warning,
            messages: [LocalizedStringKey(message.joinedParagraphs())],
            actions: [
                MullvadAlert.Action(
                    type: .default,
                    title: "Cancel",
                    handler: {
                        completion()
                    }
                ),
                MullvadAlert.Action(
                    type: .danger,
                    title: "Yes, continue",
                    identifier: .includeAllNetworksNotificationsAlertDismissButton,
                    handler: {
                        setValue(enabled)
                        completion()
                    }
                ),
            ]
        )
    }
}

// MARK: Mock

class MockIncludeAllNetworksTunnelSettingsViewModel: IncludeAllNetworksSettingsViewModel {
    var includeAllNetworksState: InclueAllNetworksState
    var localNetworkSharingState: LocalNetworkSharingState
    var consent: Bool

    var showEnableNotificationsAlert = false
    var showReconsiderNotificationsAlert = false
    var tunnelIsSecured = false

    init(
        settings: IncludeAllNetworksSettings = IncludeAllNetworksSettings(),
        appPreferences: AppPreferencesDataSource = AppPreferences()
    ) {
        includeAllNetworksState = settings.includeAllNetworksState
        localNetworkSharingState = settings.localNetworkSharingState
        consent = appPreferences.includeAllNetworksConsent
    }
}
