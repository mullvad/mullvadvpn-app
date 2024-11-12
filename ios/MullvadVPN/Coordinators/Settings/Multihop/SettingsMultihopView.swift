//
//  SettingsMultihopView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-09-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SettingsMultihopView: View {
    @State private var enabled = true

    var didToggleEnabled: ((Bool) -> Void)?

    var body: some View {
        ScrollView {
            VStack(spacing: 24) {
                SettingsInfoView(
                    viewModel: SettingsInfoViewModel(
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
                )

                SwitchRowView(
                    enabled: enabled,
                    text: NSLocalizedString(
                        "SETTINGS_SWITCH_MULTIHOP",
                        tableName: "Settings",
                        value: "Enable",
                        comment: ""
                    ),
                    didToggle: { didToggleEnabled?($0) }
                )

                Spacer()
            }
            .padding(UIMetrics.contentInsets.toEdgeInsets)
        }
        .background(Color(.secondaryColor))
        .foregroundColor(Color(.primaryTextColor))
    }
}

#Preview {
    SettingsMultihopView { enabled in
        print("\(enabled)")
    }
}
