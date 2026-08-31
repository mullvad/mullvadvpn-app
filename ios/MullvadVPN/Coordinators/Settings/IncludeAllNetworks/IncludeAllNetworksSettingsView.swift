// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import SwiftUI

@MainActor
struct IncludeAllNetworksSettingsView<ViewModel: IncludeAllNetworksSettingsViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    var showIssuesInfo: (() -> Void)?
    @State private var alert: MullvadAlert?
    private let itemFactory = SegmentedListItemFactory()

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                SettingsInfoView {
                    SettingsInfoPageView(
                        text: NSLocalizedString(
                            "Forces all app traffic on the device into the VPN tunnel, ensuring that other apps can’t "
                                + "accidentally or maliciously leak data. Apple system apps and services necessary "
                                + "for device functionality are not affected.",
                            comment: ""
                        ),
                        image: .ianOnIllustration
                    ) {
                        Text(
                            NSLocalizedString(
                                "Please swipe through and read all information in order to activate this feature",
                                comment: ""
                            )
                        )
                        .font(.mullvadTinySemiBold)
                    }
                    SettingsInfoPageView(
                        text: [
                            NSLocalizedString(
                                "If this is not enabled, malicious apps on your device can leak traffic outside the tunnel.",
                                comment: ""
                            ),
                            NSLocalizedString(
                                "Due to iOS limitations, this is not enabled by default. Our other apps tunnel all traffic via the VPN by default.",
                                comment: ""
                            ),
                        ].joinedParagraphs(),
                        image: .ianOffIllustration
                    ) {
                        let blogUrl = URL(
                            string: "https://\(ApplicationConfiguration.hostName)"
                                + "/blog/why-we-still-dont-use-includeallnetworks"
                        )!
                        ExternalLinkView(
                            url: blogUrl,
                            label: NSLocalizedString(
                                "For details, please see our blog post",
                                comment: ""
                            ),
                            font: .mullvadTiny
                        )
                    }
                    SettingsInfoPageView(
                        text: [
                            NSLocalizedString(
                                "Because of these iOS limitations, you will lose network connectivity if Mullvad VPN is "
                                    + "updated when this is enabled and you are connected to the VPN. Network connectivity "
                                    + "can only be restored by rebooting the device.",
                                comment: ""
                            ),
                            NSLocalizedString(
                                "Be cautious when using automatic updates as this will trigger the network connectivity loss.",
                                comment: ""
                            ),
                        ].joinedParagraphs(),
                        image: .ianBugIllustration
                    )
                    SettingsInfoPageView(
                        text: NSLocalizedString(
                            "Currently there is no way to work around this behaviour, but you can avoid losing network "
                                + "connectivity by disabling this feature or disconnecting before updating Mullvad VPN.",
                            comment: ""
                        ),
                        image: .ianSolutionIllustration
                    ) {
                        ActionBox(
                            isChecked: $viewModel.consent,
                            toggleTitle: "I understand the benefits and risks of using this feature"
                        )
                        .disabled(viewModel.consent)
                        .accessibilityIdentifier(.actionBox)
                    }
                }

                VStack(spacing: 0) {
                    SegmentedListItem(
                        isLastInList: false,
                        userInteraction: .enabledWithoutHighlight,
                        accessibilityIdentifier: .includeAllNetworksSwitch,
                        leading: {
                            itemFactory.leading(for: .generic(title: NSLocalizedString("Enable", comment: "")))
                        },
                        trailing: {
                            itemFactory.trailing(
                                for: .toggle(
                                    isOn: includeAllNetworksIsEnabled,
                                    isDisabled: !viewModel.consent
                                )
                            )
                        },
                        groupedContent: {
                            SegmentedListItem(
                                userInteraction: .enabledWithoutHighlight,
                                accessibilityIdentifier: .localNetworkSharingSwitch,
                                leading: {
                                    itemFactory.leading(
                                        for: .generic(title: NSLocalizedString("Local network sharing", comment: ""))
                                    )
                                },
                                trailing: {
                                    itemFactory.trailing(
                                        for: .custom(items: [
                                            .button(
                                                icon: .info,
                                                onSelect: {
                                                    alert = viewModel.getLanSharingInfoAlert {
                                                        alert = nil
                                                    }
                                                },
                                                sizing: .button
                                            ),
                                            .toggle(
                                                isOn: localNetworkSharingIsEnabled,
                                                isDisabled: !includeAllNetworksIsEnabled.wrappedValue
                                                    || !viewModel.consent
                                            ),
                                            .padding(),
                                        ])
                                    )
                                },
                                footer: MullvadInfoView(
                                    bodyText:
                                        NSLocalizedString(
                                            "Some iOS features are known to be affected when using Force all apps.",
                                            comment: ""
                                        ) + " ",
                                    link: NSLocalizedString("About known issues...", comment: ""),
                                    onTapLink: showIssuesInfo
                                )
                            )
                        }
                    )
                }
                .padding(.leading, UIMetrics.contentInsets.left)
                .padding(.trailing, UIMetrics.contentInsets.right)
            }
        }
        .onChange(of: viewModel.shouldShowEnableNotificationsAlert, initial: false) { _, showAlert in
            guard showAlert else { return }
            showEnableNotificationsAlert()
        }
        .onChange(of: viewModel.shouldShowReconsiderNotificationsAlert, initial: false) { _, showAlert in
            guard showAlert else { return }
            showReconsiderNotificationsAlert()
        }
        .mullvadAlert(item: $alert)
    }
}

#Preview {
    IncludeAllNetworksSettingsView(viewModel: MockIncludeAllNetworksTunnelSettingsViewModel())
}

// MARK: Alerts

extension IncludeAllNetworksSettingsView {
    private var includeAllNetworksIsEnabled: Binding<Bool> {
        Binding<Bool>(
            get: {
                viewModel.includeAllNetworksState.isEnabled
            },
            set: { enabled in
                alert = viewModel.getEnableFeatureAlert(feature: .includeAllNetworks, enabled: enabled) {
                    alert = nil
                }
            }
        )
    }

    private var localNetworkSharingIsEnabled: Binding<Bool> {
        Binding<Bool>(
            get: {
                viewModel.localNetworkSharingState.isEnabled
                    && viewModel.includeAllNetworksState.isEnabled
            },
            set: { enabled in
                alert = viewModel.getEnableFeatureAlert(feature: .localNetworkSharing, enabled: enabled) {
                    alert = nil
                }
            }
        )
    }

    func showEnableNotificationsAlert() {
        // Enabling IAN will result in an alert. To avoid showing
        // two alerts with no delay between them, add a short delay
        // here.
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            alert = viewModel.getEnableNotificationsAlert {
                alert = nil
            }
        }
    }

    func showReconsiderNotificationsAlert() {
        // Enabling IAN will result in an alert. To avoid showing
        // two alerts with no delay between them, add a short delay
        // here.
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            alert = viewModel.getReconsiderNotificationsAlert {
                alert = nil
            }
        }
    }
}
