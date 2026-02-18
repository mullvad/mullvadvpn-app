//
//  IncludeAllNetworksSettingsView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

@MainActor
struct IncludeAllNetworksSettingsView<ViewModel: IncludeAllNetworksSettingsViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var alert: MullvadAlert?

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                SettingsInfoView(viewModel: dataViewModel)

                VStack {
                    GroupedRowView {
                        SwitchRowView(
                            isOn: includeAllNetworksIsEnabled,
                            disabled: !viewModel.consent,
                            text: NSLocalizedString("Enable", comment: ""),
                            accessibilityId: .includeAllNetworksSwitch
                        )
                        RowSeparator(edgeInsets: .init(top: 0, leading: 16, bottom: 0, trailing: 16))
                        SwitchRowView(
                            isOn: localNetworkSharingIsEnabled,
                            disabled: !includeAllNetworksIsEnabled.wrappedValue || !viewModel.consent,
                            text: NSLocalizedString("Local network sharing", comment: ""),
                            accessibilityId: .localNetworkSharingSwitch
                        ) {
                            alert = viewModel.getLanSharingInfoAlert {
                                alert = nil
                            }
                        }
                    }
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

// MARK: Data

extension IncludeAllNetworksSettingsView {
    private var dataViewModel: SettingsInfoViewModel {
        let blogUrl = URL(
            string: "https://\(ApplicationConfiguration.hostName)/"
                + "\(ApplicationLanguage.currentLanguage.id)"
                + "/blog/why-we-still-dont-use-includeallnetworks"
        )!

        return SettingsInfoViewModel(
            pages: [
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "Forces all app traffic on the device into the VPN tunnel, ensuring that other apps can’t "
                            + "accidentally or maliciously leak data. Apple system apps and services necessary "
                            + "for device functionality are not affected.",
                        comment: ""
                    ),
                    image: .ianOnIllustration,
                    customView: AnyView(
                        Text(
                            NSLocalizedString(
                                "Please swipe through and read all information in order to activate this feature",
                                comment: ""
                            )
                        )
                        .font(.mullvadTinySemiBold)
                    )
                ),
                SettingsInfoViewModelPage(
                    body: [
                        NSLocalizedString(
                            "If this is not enabled, malicious apps on your device can leak traffic outside the tunnel.",
                            comment: ""
                        ),
                        NSLocalizedString(
                            "Due to iOS limitations, this not enabled by default. Our other apps tunnel all traffic via "
                                + "the VPN by default.",
                            comment: ""
                        ),
                    ].joinedParagraphs(),
                    image: .ianOffIllustration,
                    customView: AnyView(
                        ExternalLinkView(
                            url: blogUrl,
                            label: "For details, please see our blog post",
                            font: .mullvadTiny
                        )
                    )
                ),
                SettingsInfoViewModelPage(
                    body: [
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
                ),
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "Currently there is no way to work around this behaviour, but you can avoid losing network "
                            + "connectivity by disabling this feature or disconnecting before updating Mullvad VPN.",
                        comment: ""
                    ),
                    image: .ianSolutionIllustration,
                    customView: AnyView(
                        ActionBox(
                            isChecked: viewModel.consent,
                            label: NSLocalizedString(
                                "I understand the benefits and risks of using this feature",
                                comment: ""
                            ),
                            toggleStyle: IncludeAllNetworksCheckboxToggleStyle(),
                            didToggle: { isChecked in
                                viewModel.consent = isChecked
                            }
                        )
                        .accessibilityIdentifier(.actionBox)
                    )
                ),
            ]
        )
    }
}
