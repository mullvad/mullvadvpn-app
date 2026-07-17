//
//  RelayFilterView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-06-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadREST
import MullvadSettings
import SwiftUI

struct RelayFilterView: View {
    @ObservedObject var viewModel: RelayFilterSelection.ViewModel
    let itemFactory = SegmentedListItemFactory()

    var body: some View {
        VStack(spacing: 16) {
            RelayFilterSelection.SettingsView(viewModel: viewModel)

            Group {
                ScrollView {
                    VStack(spacing: 24) {
                        VStack(spacing: 0) {
                            filterSection(
                                title: NSLocalizedString("Ownership", comment: ""),
                                items: viewModel.ownershipItems,
                                isSelected: { .constant($0.isSelected) }
                            )
                        }

                        LazyVStack(spacing: 0) {
                            filterSection(
                                title: NSLocalizedString("Providers", comment: ""),
                                items: viewModel.providerItems,
                                isSelected: { item in
                                    Binding(
                                        get: { item.isSelected },
                                        set: { _ in viewModel.toggleItem(item) }
                                    )
                                }
                            )
                        }
                    }
                }
                .padding(.top, viewModel.featureFilters.isEmpty ? -8 : 0)

                let relayCount = viewModel.availableRelays.count
                let relayCountMessage: LocalizedStringKey =
                    if relayCount > 99 {
                        "Show \("99+") servers"
                    } else if relayCount == 0 {
                        "No matching servers"
                    } else {
                        "Show \(relayCount) servers"
                    }

                MainButton(
                    text: relayCountMessage,
                    style: .success
                ) {
                    viewModel.applyFilter()
                    viewModel.onApplyFilter?(viewModel.relayFilter)
                }
                .accessibilityIdentifier(.applyFilterButton)
                .disabled(relayCount == 0)
                .padding(.bottom, 24)
            }
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
        .background(Color.mullvadDarkBackground)
        .navigationTitle(viewModel.multihopContext == .entry ? "Entry filter" : "Exit filter")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(
                placement: .topBarTrailing,
                content: {
                    Button("Cancel") {
                        viewModel.onCancel?()
                    }
                    .foregroundStyle(Color.mullvadTextPrimary)
                }
            )
        }
    }

    @ViewBuilder
    private func filterSection(
        title: String,
        items: [RelayFilterItem],
        isSelected: @escaping (RelayFilterItem) -> Binding<Bool>
    ) -> some View {
        SegmentedListItem(
            isLastInList: false,
            leading: {
                itemFactory.leading(for: .generic(title: title))
            },
            groupedContent: {
                ForEach(Array(items.enumerated()), id: \.offset) { index, item in
                    SegmentedListItem(
                        level: 1,
                        isLastInList: item == items.last,
                        accessibilityIdentifier: item.type.accessibilityIdentifier,
                        leading: {
                            itemFactory.leading(
                                for: .relayFilter(
                                    item: item,
                                    isSelected: isSelected(item)
                                )
                            )
                        },
                        onSelect: {
                            viewModel.toggleItem(item)
                        }
                    )
                }
            }
        )
    }
}

#Preview {
    RelayFilterView(
        viewModel: .init(
            tunnelManager: RelayFilterSelection.ViewModel.MockTunnelManager(
                settings: LatestTunnelSettings(daita: DAITASettings(daitaState: .off))
            ),
            relaySelectorWrapper: RelaySelectorWrapper(relayCache: RelaySelectorStub.nonFallible().relayCache),
            multihopContext: .entry
        )
    )
}
