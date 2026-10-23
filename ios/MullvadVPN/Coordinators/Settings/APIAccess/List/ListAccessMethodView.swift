// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadREST
import MullvadSettings
import SwiftUI

struct ListAccessMethodView: View {
    private enum ActiveSheet: Identifiable {
        var id: Self { self }
        case about
        case addNewMethod
    }

    private let factory: any ListAccessMethodViewFactoryProtocol
    private let itemFactory = SegmentedListItemFactory()

    @State private var presentedSheet: ActiveSheet? = nil
    @State private var viewModel: any ListAccessViewModelProtocol
    @State private var selectedMethod: ListAccessMethodItem?

    init(factory: any ListAccessMethodViewFactoryProtocol) {
        _viewModel = State(initialValue: factory.makeListViewModel())
        self.factory = factory
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                infoHeader
                methodList
                addButton
            }
            .accessibilityIdentifier(
                AccessibilityIdentifier.apiAccessListView.asString
            )
            .padding(16.0)
        }
        .sheet(
            item: $presentedSheet,
            content: { id in
                switch id {
                case .about:
                    aboutSheet
                case .addNewMethod:
                    addNewMethodSheet
                }
            }
        )
        .background(Color.mullvadBackground)
        .navigationTitle("API access")
        .accessibilityIdentifier(.apiAccessView)
    }

    private var infoHeader: some View {
        InlineLinkText(
            text: viewModel.infoHeaderConfig.body,
            linkTitle: viewModel.infoHeaderConfig.link
        ) {
            presentedSheet = .about
        }
        .padding(.bottom, 16.0)
    }

    private var methodList: some View {
        ForEach(Array(viewModel.items.enumerated()), id: \.element.id) { index, item in
            makeListItem(item, at: index)
                .environment(\.isNestedInSegmentedListItem, index > 0)
                .padding(.bottom, 1.0)
        }
    }

    private var addButton: some View {
        MullvadButton(
            text: LocalizedStringKey("Add"),
            style: .primary
        ) {
            presentedSheet = .addNewMethod
        }
        .padding(.vertical, 24.0)
        .accessibilityIdentifier(
            AccessibilityIdentifier.addAccessMethodButton.asString
        )
    }

    private func makeListItem(
        _ item: ListAccessMethodItem,
        at index: Int
    ) -> some View {
        SegmentedListItem(
            isLastInList: index == viewModel.items.count - 1,
            accessibilityIdentifier: item.accessibilityIdentifier,
            leadingAndTrailingDestination: {
                if selectedMethod == item, let method = viewModel.fetch(item) {
                    EditAccessMethodView(
                        viewModel: factory.makeEditMethodViewModel(for: method),
                        editAccessMethodViewModel: factory.makeConfigureAccessMethodViewModel(for: method))
                } else {
                    EmptyView()
                }
            },
            leading: {
                if let detail = item.detail, !detail.isEmpty {
                    itemFactory.leading(for: .generic(title: item.name, subtitle: detail))
                } else {
                    itemFactory.leading(for: .generic(title: item.name))
                }
            },
            trailing: {
                itemFactory.trailing(
                    for: .drillDown(
                        title: viewModel.getStatus(for: item)?.description ?? "",
                        breadcrumb: nil
                    )
                )
            },
            onSelect: {
                selectedMethod = item
            })
    }

    private var aboutSheet: some View {
        NavigationStack {
            AboutView(
                header: viewModel.infoModalConfig.header,
                preamble: viewModel.infoModalConfig.preamble,
                paragraphs: viewModel.infoModalConfig.body.map {
                    AttributedString($0)
                }
            )
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        presentedSheet = nil
                    }
                }
            }
        }
        .tint(Color.mullvadTextPrimary)
    }

    private var addNewMethodSheet: some View {
        NavigationStack {
            ConfigureApiAccessMethodView(viewModel: factory.makeAddMethodViewModel())
        }
        .tint(Color.mullvadTextPrimary)
    }
}
