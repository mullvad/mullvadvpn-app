//
//  SettingsDAITAView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct SettingsDAITAView<ViewModel>: View where ViewModel: TunnelSettingsObservable<DAITASettings> {
    @StateObject var tunnelViewModel: ViewModel

    var daitaState: Binding<Bool> {
        Binding<Bool>(
            get: {
                tunnelViewModel.value.daitaState.isEnabled
            }, set: {
                var settings = tunnelViewModel.value
                settings.daitaState.isEnabled = $0

                tunnelViewModel.evaluate(setting: settings)
            }
        )
    }

    var directOnlyState: Binding<Bool> {
        Binding<Bool>(
            get: {
                tunnelViewModel.value.directOnlyState.isEnabled
            }, set: {
                var settings = tunnelViewModel.value
                settings.directOnlyState.isEnabled = $0

                tunnelViewModel.evaluate(setting: settings)
            }
        )
    }

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                SettingsInfoView(viewModel: dataViewModel)

                VStack {
                    GroupedRowView {
                        SwitchRowView(
                            enabled: daitaState,
                            text: NSLocalizedString(
                                "SETTINGS_SWITCH_DAITA_ENABLE",
                                tableName: "Settings",
                                value: "Enable",
                                comment: ""
                            )
                        )
                        .accessibilityIdentifier(AccessibilityIdentifier.daitaSwitch.rawValue)
                        RowSeparator()
                        SwitchRowView(
                            enabled: directOnlyState,
                            text: NSLocalizedString(
                                "SETTINGS_SWITCH_DAITA_DIRECT_ONLY",
                                tableName: "Settings",
                                value: "Direct only",
                                comment: ""
                            )
                        )
                        .accessibilityIdentifier(AccessibilityIdentifier.daitaDirectOnlySwitch.rawValue)
                    }

                    SettingsRowViewFooter(
                        text: NSLocalizedString(
                            "SETTINGS_SWITCH_DAITA_ENABLE",
                            tableName: "Settings",
                            value: """
                            By enabling "Direct only" you will have to manually select a server that \
                            is DAITA-enabled. Multihop won't automatically be used to enable DAITA with \
                            any server.
                            """,
                            comment: ""
                        )
                    )
                }
                .padding(.leading, UIMetrics.contentInsets.left)
                .padding(.trailing, UIMetrics.contentInsets.right)
            }
        }
        .accessibilityIdentifier(AccessibilityIdentifier.daitaView.rawValue)
    }
}

#Preview {
    SettingsDAITAView(tunnelViewModel: MockDAITATunnelSettingsViewModel())
}

extension SettingsDAITAView {
    private var dataViewModel: SettingsInfoViewModel {
        SettingsInfoViewModel(
            pages: [
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "SETTINGS_INFO_DAITA_PAGE_1",
                        tableName: "Settings",
                        value: """
                        DAITA (Defense against AI-guided Traffic Analysis) hides patterns in \
                        your encrypted VPN traffic.

                        By using sophisticated AI it’s possible to analyze the traffic of data \
                        packets going in and out of your device (even if the traffic is encrypted).

                        If an observer monitors these data packets, DAITA makes it significantly \
                        harder for them to identify which websites you are visiting or with whom \
                        you are communicating.
                        """,
                        comment: ""
                    ),
                    image: .daitaOffIllustration
                ),
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "SETTINGS_INFO_DAITA_PAGE_2",
                        tableName: "Settings",
                        value: """
                        DAITA does this by carefully adding network noise and making all network \
                        packets the same size.

                        Not all our servers are DAITA-enabled. Therefore, we use multihop \
                        automatically to enable DAITA with any server.

                        Attention: Be cautious if you have a limited data plan as this feature \
                        will increase your network traffic.
                        """,
                        comment: ""
                    ),
                    image: .daitaOnIllustration
                ),
            ]
        )
    }
}
