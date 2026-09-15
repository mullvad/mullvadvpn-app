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

struct SettingsView: View {
    @ObservedObject var viewModel: SettingsViewModel
    let onSelect: @MainActor (SettingsNavigationRoute) -> Void
    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: SettingsViewModel, onSelect: @escaping @MainActor (SettingsNavigationRoute) -> Void) {
        self.viewModel = viewModel
        self.onSelect = onSelect
    }

    var body: some View {
        SettingsInfoContainerView {
            VStack(spacing: UIMetrics.TableView.sectionSpacing) {
                ForEach(Array(viewModel.sections.enumerated()), id: \.offset) { _, section in
                    sectionView(section)
                }
            }
        }
        .onAppear {
            viewModel.refresh()
        }
        .navigationTitle("Settings")
    }

    @ViewBuilder
    private func sectionView(_ section: SettingsViewModel.Section) -> some View {
        if let first = section.rows.first {
            let rest = Array(section.rows.dropFirst())
            VStack(alignment: .leading, spacing: 1) {
                if rest.isEmpty {
                    rowView(first, isLastInList: true)
                } else {
                    rowView(first, isLastInList: false) {
                        ForEach(Array(rest.enumerated()), id: \.element.route) { index, row in
                            rowView(row, isLastInList: index == rest.count - 1)
                        }
                    }
                }
                if let footer = section.footer {
                    SettingsRowViewFooter(text: footer)
                }
            }
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
    }

    private func rowView<GroupedContent: View>(
        _ row: SettingsViewModel.Row,
        isLastInList: Bool,
        @ViewBuilder groupedContent: () -> GroupedContent = { EmptyView() }
    ) -> some View {
        SegmentedListItem(
            isLastInList: isLastInList,
            accessibilityIdentifier: row.accessibilityIdentifier,
            leading: {
                itemFactory.leading(for: .generic(title: row.title, subtitle: row.subtitle))
            },
            trailing: {
                if row.isExternal {
                    itemFactory.trailing(for: .external(title: row.detail))
                } else {
                    itemFactory.trailing(for: .drillDown(title: row.detail, breadcrumb: row.breadcrumb))
                }
            },
            groupedContent: groupedContent,
            onSelect: {
                onSelect(row.route)
            }
        )
    }
}
