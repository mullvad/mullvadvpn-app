//
//  SettingsMultihopView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct SettingsMultihopView<ViewModel>: View where ViewModel: TunnelSettingsObservable<MultihopState> {
    @StateObject var tunnelViewModel: ViewModel

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                SettingsInfoView(viewModel: dataViewModel)

                SwitchRowView(
                    enabled: $tunnelViewModel.value.isEnabled,
                    text: NSLocalizedString(
                        "SETTINGS_SWITCH_MULTIHOP",
                        tableName: "Settings",
                        value: "Enable",
                        comment: ""
                    )
                )
                .padding(.leading, UIMetrics.contentInsets.left)
                .padding(.trailing, UIMetrics.contentInsets.right)
                .accessibilityIdentifier(AccessibilityIdentifier.multihopSwitch.rawValue)
            }
        }.accessibilityIdentifier(AccessibilityIdentifier.multihopView.rawValue)
    }
}

#Preview {
    SettingsMultihopView(tunnelViewModel: MockMultihopTunnelSettingsViewModel())
}

extension SettingsMultihopView {
    private var dataViewModel: SettingsInfoViewModel {
        SettingsInfoViewModel(
            pages: [
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "SETTINGS_INFO_MULTIHOP",
                        tableName: "Settings",
                        value: """
                        Multihop routes your traffic into one WireGuard server and out another, making it \
                        harder to trace. This results in increased latency but increases anonymity online.
                        """,
                        comment: ""
                    ),
                    image: .multihopIllustration
                ),
            ]
        )
    }
}
