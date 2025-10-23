//
//  SettingsDAITAView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct SettingsDAITAView<ViewModel>: View where ViewModel: TunnelSettingsObservable<DAITASettings> {
    @StateObject var tunnelViewModel: ViewModel

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                if isAutomaticRoutingActive {
                    DAITAMultihopNotice()
                        .padding(
                            EdgeInsets(
                                top: -8,
                                leading: UIMetrics.contentInsets.toEdgeInsets.leading,
                                bottom: 8,
                                trailing: UIMetrics.contentInsets.toEdgeInsets.trailing
                            ))
                }

                SettingsInfoView(viewModel: dataViewModel)

                VStack {
                    GroupedRowView {
                        SwitchRowView(
                            isOn: daitaIsEnabled,
                            text: NSLocalizedString("Enable", comment: ""),
                            accessibilityId: .daitaSwitch
                        )
                        RowSeparator(edgeInsets: .init(top: 0, leading: 16, bottom: 0, trailing: 16))
                        SwitchRowView(
                            isOn: directOnlyIsEnabled,
                            disabled: !daitaIsEnabled.wrappedValue,
                            text: NSLocalizedString("Direct only", comment: ""),
                            accessibilityId: .daitaDirectOnlySwitch
                        )
                    }

                    SettingsRowViewFooter(
                        text: String(
                            format:
                                NSLocalizedString(
                                    "By enabling “%@” you will have to manually select a server that is %@-enabled. "
                                        + "%@ won't automatically be used to enable DAITA with any server.",
                                    comment: ""
                                ),
                            NSLocalizedString("Direct only", comment: ""),
                            NSLocalizedString("DAITA", comment: ""),
                            NSLocalizedString("Multihop", comment: "")
                        )
                    )
                }
                .padding(.leading, UIMetrics.contentInsets.left)
                .padding(.trailing, UIMetrics.contentInsets.right)
            }
        }
    }
}

#Preview {
    SettingsDAITAView(tunnelViewModel: MockDAITATunnelSettingsViewModel())
}

extension SettingsDAITAView {
    var daitaIsEnabled: Binding<Bool> {
        Binding<Bool>(
            get: {
                tunnelViewModel.value.daitaState.isEnabled
            },
            set: { enabled in
                var settings = tunnelViewModel.value
                settings.daitaState.isEnabled = enabled

                tunnelViewModel.evaluate(setting: settings)
            }
        )
    }

    var directOnlyIsEnabled: Binding<Bool> {
        Binding<Bool>(
            get: {
                tunnelViewModel.value.directOnlyState.isEnabled
            },
            set: { enabled in
                var settings = tunnelViewModel.value
                settings.directOnlyState.isEnabled = enabled

                tunnelViewModel.evaluate(setting: settings)
            }
        )
    }

    var isAutomaticRoutingActive: Bool {
        let viewModel = tunnelViewModel as? DAITATunnelSettingsViewModel
        return viewModel?.isAutomaticRoutingActive ?? false
    }
}

extension SettingsDAITAView {
    private var dataViewModel: SettingsInfoViewModel {
        let daitafullTitle = "Defense against AI-guided Traffic Analysis"
        let daitaTitle = NSLocalizedString("DAITA", comment: "")
        return SettingsInfoViewModel(
            pages: [
                SettingsInfoViewModelPage(
                    body: [
                        NSLocalizedString(
                            "**Attention: This increases network traffic and will also negatively affect "
                                + "speed, latency, and battery usage. Use with caution on limited plans.**",
                            comment: ""
                        ),
                        String(
                            format: NSLocalizedString(
                                "%@ (%@) hides patterns in your encrypted VPN traffic.",
                                comment: ""
                            ),
                            daitaTitle,
                            daitafullTitle
                        ),
                        NSLocalizedString(
                            "By using sophisticated AI it’s possible to analyze "
                                + "the traffic of data packets going in and out of your "
                                + "device (even if the traffic is encrypted).",
                            comment: ""
                        ),
                    ].joinedParagraphs(),
                    image: .daitaOffIllustration
                ),
                SettingsInfoViewModelPage(
                    body: [
                        String(
                            format: NSLocalizedString(
                                "If an observer monitors these data packets, %@ makes it "
                                    + "significantly harder for them to identify which websites "
                                    + "you are visiting or with whom you are communicating.",
                                comment: ""
                            ), daitaTitle),
                        String(
                            format: NSLocalizedString(
                                "%@ does this by carefully adding network noise and making "
                                    + "all network packets the same size.",
                                comment: ""
                            ), daitaTitle),
                        String(
                            format: NSLocalizedString(
                                "Not all our servers are %@-enabled. Therefore, we use multihop "
                                    + "automatically to enable %@ with any server.",
                                comment: ""
                            ), daitaTitle, daitaTitle),
                    ].joinedParagraphs(),
                    image: .daitaOnIllustration
                ),
            ]
        )
    }
}
