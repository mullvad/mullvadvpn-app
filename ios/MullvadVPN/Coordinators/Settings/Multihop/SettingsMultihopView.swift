//
//  SettingsMultihopView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct SettingsMultihopView: View {
    @StateObject var viewModel: MultihopTunnelSettingsViewModel
    @State private var alert: MullvadAlert?
    private let itemFactory = ListItemFactory()

    private struct OptionSpec: Identifiable {
        let id: MultihopState
        let label: String
        let accessibilityIdentifier: AccessibilityIdentifier
        let customView: AnyView?
    }

    private let options: [OptionSpec] = [
        .init(
            id: .whenNeeded,
            label: MultihopState.whenNeeded.description,
            accessibilityIdentifier: .multihopState(MultihopState.whenNeeded.description),
            customView: AnyView(WhenNeededAlert())
        ),
        .init(
            id: .always,
            label: MultihopState.always.description,
            accessibilityIdentifier: .multihopState(MultihopState.always.description),
            customView: nil
        ),
        .init(
            id: .never,
            label: MultihopState.never.description,
            accessibilityIdentifier: .multihopState(MultihopState.never.description),
            customView: nil
        ),
    ]

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 8) {
                if viewModel.automaticRoutingIsActive {
                    AutomaticLocationNotice()
                        .padding(
                            EdgeInsets(
                                top: 0,
                                leading: UIMetrics.contentInsets.toEdgeInsets.leading,
                                bottom: 16,
                                trailing: UIMetrics.contentInsets.toEdgeInsets.trailing
                            )
                        )
                }

                SettingsInfoView(viewModel: dataViewModel)

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
                                                    viewModel.multihopState == option.id
                                            ))
                                    },
                                    segment: {
                                        if let customView = option.customView {
                                            itemFactory.segment(
                                                for: .info(onSelect: {
                                                    alert = getInfoAlert(for: customView) { alert = nil }
                                                })
                                            )
                                        }
                                    },
                                    groupedContent: {},
                                    onSelect: {
                                        viewModel.multihopState = option.id
                                    }
                                )
                            }
                        },
                        onSelect: {}
                    )
                }
                .padding(.leading, UIMetrics.contentInsets.left)
                .padding(.trailing, UIMetrics.contentInsets.right)
            }
        }
        .mullvadAlert(item: $alert)
    }

    private func getInfoAlert(for customView: AnyView, completion: @escaping () -> Void) -> MullvadAlert {
        MullvadAlert(
            type: .info,
            customView: customView,
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

extension SettingsMultihopView {
    private var dataViewModel: SettingsInfoViewModel {
        SettingsInfoViewModel(
            pages: [
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "Multihop routes your traffic into one WireGuard server and out another, "
                            + "making it harder to trace. This results in increased latency but increases "
                            + "anonymity online. Multihop has three different modes to choose between: "
                            + "When needed, Always, and Never.",
                        comment: ""
                    ),
                    image: .multihopIllustrationGeneral
                ),
                SettingsInfoViewModelPage(
                    image: .multihopIllustrationWhenNeeded,
                    customView: AnyView(WhenNeededPage())
                ),
                SettingsInfoViewModelPage(
                    image: .multihopIllustrationAlways,
                    customView: AnyView(AlwaysPage())
                ),
                SettingsInfoViewModelPage(
                    image: .multihopIllustrationNever,
                    customView: AnyView(NeverPage())
                ),
            ]
        )
    }

    private struct WhenNeededPage: View {
        var body: some View {
            VStack(alignment: .leading) {
                Text("When needed")
                    .fontWeight(.bold)
                Text(
                    "To ensure your current settings work with your selected location, and to "
                        + "avoid blocking your connection, the app might automatically multihop via "
                        + "a different entry server."
                )
                Text(
                    "This will be indicated by the \(UIImage.Multihop.whenNeeded.scaledIcon(fromBaseSize: 14, to: .subheadline, offset: .init(x: 0, y: 2))) symbol"
                )
            }
            .font(.mullvadTiny)
            .foregroundStyle(Color.mullvadTextSecondary)
        }
    }

    private struct AlwaysPage: View {
        var body: some View {
            VStack(alignment: .leading) {
                Text("Always")
                    .fontWeight(.bold)
                Text(
                    "Multihop is enabled. Your connection is routed through an entry server before "
                        + "exiting through the selected location."
                )
            }
            .font(.mullvadTiny)
            .foregroundStyle(Color.mullvadTextSecondary)
        }
    }

    private struct NeverPage: View {
        var body: some View {
            VStack(alignment: .leading) {
                Text("Never")
                    .fontWeight(.bold)
                Text(
                    "Multihop is disabled. Your selected location must support all active settings in "
                        + "order to establish a connection."
                )
            }
            .font(.mullvadTiny)
            .foregroundStyle(Color.mullvadTextSecondary)
        }
    }

    private struct WhenNeededAlert: View {
        var body: some View {
            VStack(alignment: .leading, spacing: 16) {
                Text(
                    "To ensure your current settings work with your selected location, and to "
                        + "avoid blocking your connection, the app might automatically multihop via "
                        + "a different entry server."
                )
                Text(
                    "This will be indicated by the \(UIImage.Multihop.whenNeeded.scaledIcon(fromBaseSize: 15, to: .body, offset: .init(x: 0, y: 2))) symbol."
                )
                Text(
                    "Attention: This will ignore filter settings for the entry server that is "
                        + "being automatically selected."
                )
            }
            .font(.mullvadSmall)
            .foregroundStyle(Color.mullvadTextSecondary)
        }
    }

    struct AutomaticLocationNotice: View {
        var body: some View {
            HStack(alignment: .center, spacing: 8) {
                UIImage.Multihop.whenNeeded.scaledIcon(fromBaseSize: 18, to: .subheadline, offset: .init(x: 0, y: 2))
                Text("An additional server is used to match your settings for your selected location")
            }
            .font(.mullvadTinySemiBold)
            .foregroundColor(Color.mullvadTextSecondary)
        }
    }
}
