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
    let viewModel: any SettingsViewModelProtocol
    let onSelect: @MainActor (SettingsNavigationRoute) -> Void
    private let itemFactory = SegmentedListItemFactory()

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
    private func sectionView(_ section: SettingsSection) -> some View {
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
        _ row: SettingsRow,
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
                trailingView(row)
            },
            groupedContent: groupedContent,
            onSelect: {
                onSelect(row.route)
            }
        )
    }

    /// Unlike the factory's fixed-size string, the detail wraps so it shares the row width with the title.
    private func trailingView(_ row: SettingsRow) -> some View {
        HStack(spacing: 0) {
            if let breadcrumb = row.breadcrumb {
                itemFactory.trailing(for: .custom(items: [.breadcrumb(breadcrumb)]))
            }
            if !row.detail.isEmpty {
                Text(row.detail)
                    .font(.mullvadTiny)
                    .foregroundStyle(Color.mullvadTextSecondary)
                    .multilineTextAlignment(.trailing)
            }
            itemFactory.trailing(for: .custom(items: [.icon(row.isExternal ? .external : .chevron, sizing: .button)]))
        }
    }
}

#Preview {
    NavigationStack {
        SettingsView(viewModel: MockRootSettingsViewModel()) { _ in }
    }
}

#Preview("Accessibility size") {
    NavigationStack {
        SettingsView(viewModel: MockRootSettingsViewModel()) { _ in }
    }
    .environment(\.dynamicTypeSize, .accessibility3)
}
