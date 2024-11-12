//
//  SettingsMultihopView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import SwiftUI

struct SettingsMultihopView<VM>: View where VM: MultihopTunnelSettingsObservable {
    @StateObject var tunnelViewModel: VM

    private let viewModel = SettingsInfoViewModel(
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
            )
        ]
    )

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                SettingsInfoView(viewModel: viewModel)

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
            }
        }
    }
}

#Preview {
    SettingsMultihopView(tunnelViewModel: MockMultihopTunnelSettingsViewModel())
}
