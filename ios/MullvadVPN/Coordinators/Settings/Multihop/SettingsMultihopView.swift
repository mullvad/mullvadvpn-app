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
    private let itemFactory = ListItemFactory()

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
                    VStack(spacing: 0) {
                        SegmentedListItem(
                            isLastInList: false,
                            label: {
                                itemFactory.label(for: .setting(title: "Mode"))
                            },
                            segment: {},
                            groupedContent: {
                                ForEach(Array(options.enumerated()), id: \.element.id) { index, option in
                                    SegmentedListItem(
                                        level: 1,
                                        isLastInList: index == options.count - 1,
                                        accessibilityIdentifier: option.accessibilityIdentifier,
                                        label: {
                                            itemFactory.label(
                                                for: .setting(
                                                    title: option.label,
                                                    level: 1,
                                                    selected:
                                                        tunnelViewModel.value == option.id
                                                ))
                                        },
                                        segment: {
                                            if let helpText = option.helpText {
                                                itemFactory.segment(
                                                    for: .info(onSelect: {
                                                        alert = getInfoAlert(for: helpText) { alert = nil }
                                                    })
                                                )
                                            }
                                        },
                                        groupedContent: {},
                                        onSelect: {
                                            tunnelViewModel.evaluate(setting: option.id)
                                        }
                                    )
                                }
                            },
                            onSelect: {}
                        )
                    }
                    .padding(.leading, UIMetrics.contentInsets.left)
                    .padding(.trailing, UIMetrics.contentInsets.right)
                #else
                    SwitchRowView(
                        isOn: $tunnelViewModel.value.isUserSelected,
                        text: NSLocalizedString("Enable", comment: ""),
                        accessibilityId: .multihopSwitch
                    )
                    .padding(.leading, UIMetrics.contentInsets.left)
                    .padding(.trailing, UIMetrics.contentInsets.right)
                #endif
            }
        }
        .mullvadAlert(item: $alert)
    }

    private func getInfoAlert(for messages: [LocalizedStringKey], completion: @escaping () -> Void) -> MullvadAlert {
        MullvadAlert(
            type: .info,
            messages: messages,
            actions: [
                MullvadAlert.Action(
                    type: .default,
                    title: "Got it!",
                    handler: completion
                )
            ]
        )
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
