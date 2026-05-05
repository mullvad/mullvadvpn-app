//
//  MultihopMigrationWizardView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct MultihopMigrationWizardView<ViewModel: MultihopMigrationWizardViewModelProtocol>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var currentPage = 0

    var body: some View {
        VStack(spacing: 0.0) {
            PaginationView(
                pages: viewModel.items.map { stateViewModel in
                    MullvadStateView(viewModel: stateViewModel)
                }, currentPage: $currentPage)

            HStack(spacing: 8.0) {
                MainButton(text: "Back", style: .default) {
                    if currentPage > 0 {
                        currentPage -= 1
                    }
                }
                .showIf(currentPage > 0)

                MainButton(text: currentPage == viewModel.items.count - 1 ? "Got it!" : "Next", style: .default) {
                    if currentPage < viewModel.items.count - 1 {
                        currentPage += 1
                    }
                }
            }
            .padding(.horizontal, 16.0)
            .padding(.bottom, 24.0)
        }
        .background(Color.mullvadBackground)
    }
}

#Preview {
    MultihopMigrationWizardView(
        viewModel: MockMultihopMigrationWizardViewModel()
    )
}

// MARK: - Mock ViewModel

final class MockMultihopMigrationWizardViewModel: MultihopMigrationWizardViewModelProtocol {
    var items: [StateViewModel] {
        let changes: [Change] = [
            Change(path: .automatic),
            Change(path: .uniqueFilter),
            Change(path: .directOnlyRemoved),
            Change(
                path: .updatedMultiHop,
                before: MultihopStateV1.off,
                after: MultihopStateV2.whenNeeded),
        ]

        return changes.map { change in
            let descriptor = SettingsUpdateDescriptor(change: change)

            return StateViewModel(
                style: .info,
                title: TextItem(
                    text: descriptor.title,
                    style: .headline
                ),
                banner: descriptor.banner,
                details: descriptor.description
            )
        }
    }
}
