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
                ForEach(viewModel.sections, id: \.kind) { section in
                    sectionView(section)
                }
            }
        }
        .onAppear {
            viewModel.refresh()
        }
        .navigationTitle("Settings")
    }

    private func sectionView(_ section: SettingsSection) -> some View {
        VStack(alignment: .leading, spacing: 1) {
            ForEach(Array(section.rows.enumerated()), id: \.element.route) { index, row in
                rowView(row, isLastInList: index == section.rows.count - 1)
                    .environment(\.isNestedInSegmentedListItem, index > 0)
            }
            if let footer = section.kind.footer {
                SettingsRowViewFooter(text: footer)
            }
        }
        .padding(.leading, UIMetrics.contentInsets.left)
        .padding(.trailing, UIMetrics.contentInsets.right)
    }

    /// Shows the detail beside the title when the row fits on one line, otherwise below it.
    @ViewBuilder
    private func rowView(_ row: SettingsRow, isLastInList: Bool) -> some View {
        if let detail = row.detail {
            ViewThatFits(in: .horizontal) {
                rowItem(row, isLastInList: isLastInList, subtitle: row.subtitle, trailingDetail: detail)
                rowItem(row, isLastInList: isLastInList, subtitle: row.subtitle ?? detail)
            }
        } else {
            rowItem(row, isLastInList: isLastInList, subtitle: row.subtitle)
        }
    }

    private func rowItem(
        _ row: SettingsRow,
        isLastInList: Bool,
        subtitle: String?,
        trailingDetail: String? = nil
    ) -> some View {
        SegmentedListItem(
            isLastInList: isLastInList,
            accessibilityIdentifier: row.accessibilityIdentifier,
            accessibilityLabel: [row.title, row.subtitle, row.detail].compactMap { $0 }.joined(separator: ", "),
            leading: {
                itemFactory.leading(for: .generic(title: row.title, subtitle: subtitle))
            },
            trailing: {
                itemFactory.trailing(
                    for: .custom(
                        items: [
                            row.breadcrumb.map { .breadcrumb($0) },
                            trailingDetail.map { .string($0) },
                            .icon(row.isExternal ? .external : .chevron, sizing: .button),
                        ].compactMap { $0 }
                    )
                )
            },
            onSelect: {
                onSelect(row.route)
            }
        )
    }
}

#if DEBUG
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
#endif
