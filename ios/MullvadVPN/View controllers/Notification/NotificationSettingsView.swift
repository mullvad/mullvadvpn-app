//
//  NotificationSettingsView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct NotificationSettingsView<ViewModel>: View where ViewModel: NotificationSettingsViewModelProtocol {
    @ObservedObject var viewModel: ViewModel
    var didUpdateNotificationSettings: ((NotificationSettings) -> Void)?

    var body: some View {
        GeometryReader { geo in
            SettingsInfoContainerView {
                VStack(alignment: .leading) {
                    GroupedRowView {
                        ForEach(NotificationKeys.allCases, id: \.self) { key in
                            SwitchRowView(
                                isOn: viewModel.binding(for: key),
                                disabled: !viewModel.isNotificationsAllowed,
                                text: NSLocalizedString(key.title, comment: ""),
                                accessibilityId: key.identifier
                            )
                            RowSeparator()
                        }
                    }

                    Spacer()

                    Text(
                        "Notifications for Mullvad VPN are disabled on this device. Please change your system settings for Mullvad VPN if you wish to enable them again."
                    )
                    .font(.mullvadSmall)
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.white.opacity(0.6))
                    .padding(.bottom, 16)
                    .showIf(viewModel.isNotificationsDisabled)
                    MainButton(
                        text: "Enable notifications",
                        style: .default,
                        action: {
                            viewModel.enableNotifications()
                        }
                    )
                    .showIf(!viewModel.isNotificationsAllowed)

                    MainButton(
                        text: "Open system settings",
                        style: .default,
                        action: {
                            viewModel.openAppSettings()
                        }
                    )
                    .showIf(viewModel.isNotificationsDisabled)
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 24)
                .frame(minHeight: geo.size.height)

            }
        }
        .onAppear {
            viewModel.checkNotificationPermission()
        }
        .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
            viewModel.checkNotificationPermission()
        }
        .onChange(
            of: viewModel.settings,
            {
                didUpdateNotificationSettings?(viewModel.settings)
            })
    }
}

#Preview {
    NotificationSettingsView(viewModel: NotificationSettingsViewModel(settings: NotificationSettings()))
}

private extension NotificationKeys {
    var title: String {
        switch self {
        case .account:
            "Account time reminders"
        }
    }

    var identifier: AccessibilityIdentifier {
        switch self {
        case .account:
            .accountNotificationSwitch
        }
    }
}
