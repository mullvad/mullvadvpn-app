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
import MullvadTypes
import SwiftUI

struct EditAccessMethodView<ViewModel: EditAccessMethodViewModelProtocol>: View {
    @State var viewModel: ViewModel
    @State private var showDetails = false
    @State private var deleteConfirmationAlert: MullvadAlert?
    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: ViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                if let infoHeaderConfig = viewModel.infoHeaderConfig {
                    InlineLinkText(
                        text: infoHeaderConfig.body,
                        linkTitle: infoHeaderConfig.link
                    ) {
                        showDetails = viewModel.infoModalConfig != nil
                    }
                    .padding(.bottom, 16.0)
                    .sheet(isPresented: $showDetails) {
                        NavigationStack {
                            AboutView(
                                header: viewModel.infoModalConfig!.header,
                                preamble: viewModel.infoModalConfig!.preamble,
                                paragraphs: viewModel.infoModalConfig!.body.map { AttributedString.init($0) }
                            )
                            .toolbar {
                                ToolbarItem(placement: .topBarTrailing) {
                                    Button("Done") {
                                        showDetails = false
                                    }
                                }
                            }
                            .toolbarBackground(Color.mullvadBackground, for: .navigationBar)
                        }
                        .tint(.mullvadTextPrimary)
                    }
                }

                SegmentedListItem(
                    isLastInList: true,
                    userInteraction: .enabledWithoutHighlight,
                    accessibilityIdentifier: .accessMethodEnableSwitch,
                    leading: {
                        itemFactory.leading(for: .generic(title: NSLocalizedString("Enable method", comment: "")))
                    },
                    trailing: {
                        itemFactory.trailing(
                            for: .toggle(
                                isOn: $viewModel.isEnabled,
                                isDisabled: false
                            )
                        )
                    }
                )
                .padding(.bottom, 24.0)

                if viewModel.isEditable {
                    SegmentedListItem(
                        isLastInList: true,
                        userInteraction: .enabledWithoutHighlight,
                        accessibilityIdentifier: .accessMethodEnableSwitch,
                        leading: {
                            itemFactory.leading(for: .generic(title: NSLocalizedString("Edit method", comment: "")))
                        },
                        trailing: {
                            itemFactory.image(for: .chevron)
                        },
                        onSelect: {
                            //It can Be replaced with navigation link when It's fully migrated
                            viewModel.edit()
                        }
                    )
                    .padding(.bottom, 24.0)
                }

                ApiAccessMethodTestView(
                    status: viewModel.testState
                ) {
                    viewModel.startTesting()
                } onStop: {
                    viewModel.stopTesting()
                }

                if viewModel.isEditable {
                    MullvadButton(
                        text: "Delete method", style: .destructive,
                        action: {
                            deleteConfirmationAlert = MullvadAlert(
                                type: .info, messages: ["Delete \(viewModel.name)?"],
                                actions: [
                                    .init(
                                        type: .destructive, title: "Delete",
                                        identifier: AccessibilityIdentifier.accessMethodConfirmDeleteButton,
                                        handler: {
                                            deleteConfirmationAlert = nil
                                            viewModel.delete()
                                        }),
                                    .init(
                                        type: .secondary,
                                        title: "Cancel",
                                        handler: {
                                            deleteConfirmationAlert = nil
                                        }
                                    ),
                                ])
                        }
                    )
                    .padding(.vertical, 24.0)
                }
            }
        }
        .mullvadAlert(item: $deleteConfirmationAlert)
        .onDisappear(perform: {
            viewModel.stopTesting()
        })
        .scrollBounceBehavior(.automatic)
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
        .background(Color.mullvadBackground)
    }
}
