//
//  SettingsMultihopView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct SettingsMultihopView<ViewModel>: View where ViewModel: TunnelSettingsObservable<MultihopState> {
    @StateObject var tunnelViewModel: ViewModel
    @State private var alert: MullvadAlert?

    private struct OptionSpec: Identifiable {
        let id: MultihopState
        let label: String
        let accessibilityIdentifier: AccessibilityIdentifier
        let helpText: [LocalizedStringKey]?
    }

    private let options: [OptionSpec] = [
        .init(
            id: .whenNeeded,
            label: "When needed",
            accessibilityIdentifier: .multihopWhenNeeded,
            helpText: [
                "If your selected location does not support your preferences multihop will be used automatically to connect to that location via a compatible server. This will be indicated by the \(Image("IconSmartLocation")) symbol",
                "",
                "Attention: This will ignore filter settings for the entry server that is being automatically selected.",
            ]),
        .init(
            id: .always,
            label: "Always",
            accessibilityIdentifier: .multihopAlways,
            helpText: [
                "Always connect via an entry server. The location can either be set manually or automatically in the \"Select location\" view."
            ]),
        .init(
            id: .never,
            label: "Never",
            accessibilityIdentifier: .multihopNever,
            helpText: nil
        ),
    ]

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                SettingsInfoView(viewModel: dataViewModel)

                #if DEBUG
                    VStack(spacing: 1) {
                        HStack {
                            Text("Mode")
                            Spacer()
                        }
                        .padding(EdgeInsets(UIMetrics.SettingsCell.defaultLayoutMargins))
                        .background(Color(UIColor.primaryColor))
                        ForEach(options) { option in
                            HStack(spacing: 1) {
                                HStack {
                                    Image(uiImage: UIImage.tick).opacity(tunnelViewModel.value == option.id ? 1.0 : 0.0)
                                        .foregroundStyle(
                                            (tunnelViewModel.value == option.id)
                                                ? Color(UIColor.Cell.Background.selected)
                                                : Color(UIColor.Cell.titleTextColor)
                                        )
                                    Spacer().frame(width: UIMetrics.SettingsCell.selectableSettingsCellLeftViewSpacing)
                                    Text(option.label)
                                        .foregroundStyle(
                                            (tunnelViewModel.value == option.id)
                                                ? Color(UIColor.Cell.Background.selected)
                                                : Color(UIColor.Cell.titleTextColor)
                                        )
                                    Spacer()
                                }
                                .padding(EdgeInsets(UIMetrics.SettingsCell.defaultLayoutMargins))
                                .background(
                                    Color(UIColor.Cell.Background.indentationLevelZero)
                                )
                                if let helpText = option.helpText {
                                    VStack {
                                        Spacer()
                                        Button(action: {
                                            self.alert = MullvadAlert(
                                                type: .info, messages: helpText,
                                                actions: [
                                                    .init(
                                                        type: .default,
                                                        title: "Got it!",
                                                        identifier: .includeAllNetworksNotificationsAlertDismissButton,
                                                        handler: {
                                                            self.alert = nil
                                                        }
                                                    )
                                                ])
                                        }) {
                                            Image(.iconInfo)
                                        }
                                        .tint(.white)
                                        Spacer()
                                    }
                                    .padding(EdgeInsets(top: 0, leading: 16, bottom: 0, trailing: 16))
                                    .background(
                                        Color(UIColor.Cell.Background.indentationLevelZero)
                                    )
                                }
                            }
                            .foregroundColor(Color(UIColor.Cell.titleTextColor))
                            .onTapGesture {
                                tunnelViewModel.value = option.id
                            }
                            .accessibilityIdentifier(option.accessibilityIdentifier.asString)
                        }
                    }
                    .cornerRadius(16)
                    .padding(.leading, UIMetrics.contentInsets.left)
                    .padding(.trailing, UIMetrics.contentInsets.right)
                    .listStyle(.plain)
                    .listRowSpacing(UIMetrics.TableView.separatorHeight)
                    .environment(\.defaultMinListRowHeight, 0)
                    .background(Color(.secondaryColor))
                    .foregroundColor(Color(.primaryTextColor))
                #else
                    SwitchRowView(
                        isOn: $tunnelViewModel.value.isUserSelected,
                        text: NSLocalizedString("Enable", comment: ""),
                        accessibilityId: .multihopSwitch
                    )
                #endif
            }
        }
        .mullvadAlert(item: $alert)
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
                        """
                        Multihop routes your traffic into one WireGuard server and out another, making it \
                        harder to trace. This results in increased latency but increases anonymity online.
                        """,
                        comment: ""
                    ),
                    image: .multihopIllustration
                )
            ]
        )
    }
}
