//
//  SettingsMigrationWizardView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct SettingsMigrationWizardView<ViewModel: SettingsMigrationWizardViewModelProtocol>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var currentPage = 0
    var didFinish: (() -> Void)?

    var body: some View {
        VStack(spacing: 0.0) {
            MullvadPaginationView(
                pages: viewModel.items.map { stateViewModel in
                    MullvadStateView(viewModel: stateViewModel)
                }, currentPage: $currentPage
            )
            .padding(.top, 8.0)

            HStack(spacing: 8.0) {
                MullvadButton(text: "Back", style: .primary) {
                    if currentPage > 0 {
                        currentPage -= 1
                    }
                }
                .showIf(viewModel.items.count > 1)
                .disabled(currentPage == 0)

                MullvadButton(text: currentPage == viewModel.items.count - 1 ? "Got it!" : "Next", style: .primary) {
                    if currentPage < viewModel.items.count - 1 {
                        currentPage += 1
                    } else {
                        didFinish?()
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
    SettingsMigrationWizardView(
        viewModel: MockMultihopMigrationWizardViewModel()
    )
}

// MARK: - Mock ViewModel

final class MockMultihopMigrationWizardViewModel: SettingsMigrationWizardViewModelProtocol {
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
                title: MullvadStateView.TextItem(
                    text: descriptor.title,
                    style: .headline()
                ),
                banner: descriptor.banner,
                details: descriptor.description
            )
        }
    }
}
