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

struct EditAccessMethodView: View {
    private enum ActiveSheet: Identifiable {
        var id: Self { self }
        case about
    }

    @State private var presentedSheet: ActiveSheet? = nil
    @State private var deleteConfirmationAlert: MullvadAlert?
    @Environment(\.dismiss) private var dismiss

    @State private var viewModel: any EditAccessMethodViewModelProtocol
    @State private var editAccessMethodViewModel: any ConfigureApiAccessMethodViewModelProtocol

    let itemFactory = SegmentedListItemFactory()

    init(
        viewModel: any EditAccessMethodViewModelProtocol,
        editAccessMethodViewModel: any ConfigureApiAccessMethodViewModelProtocol
    ) {
        _viewModel = State(initialValue: viewModel)
        _editAccessMethodViewModel = State(initialValue: editAccessMethodViewModel)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                if let infoHeaderConfig = viewModel.infoHeaderConfig {
                    infoHeader(infoHeaderConfig)
                }

                enableMethodRow

                if viewModel.isEditable {
                    editMethodRow
                }

                testView

                if viewModel.isEditable {
                    deleteButton
                }
            }
        }
        .accessibilityIdentifier(.editAccessMethodView)
        .sheet(
            item: $presentedSheet,
            content: { id in
                switch id {
                case .about:
                    aboutSheet
                }
            }
        )
        .mullvadAlert(item: $deleteConfirmationAlert)
        .onDisappear(perform: {
            viewModel.stopTesting()
        })
        .scrollBounceBehavior(.automatic)
        .padding(.horizontal, 16)
        .background(Color.mullvadBackground)
        .navigationTitle(viewModel.name)
        .toolbar(.visible, for: .navigationBar)
    }

    private func infoHeader(_ infoHeaderConfig: InfoHeaderConfig) -> some View {
        InlineLinkText(
            text: infoHeaderConfig.body,
            linkTitle: infoHeaderConfig.link
        ) {
            presentedSheet = .about
        }
        .padding(.bottom, 16.0)
    }

    private var enableMethodRow: some View {
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
    }

    private var editMethodRow: some View {
        SegmentedListItem(
            isLastInList: true,
            leadingAndTrailingDestination: {
                ConfigureApiAccessMethodView(viewModel: editAccessMethodViewModel)
            },
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Edit method", comment: "")))
            },
            trailing: {
                itemFactory.trailing(for: .drillDown(title: "", breadcrumb: nil))
            }
        )
        .padding(.bottom, 24.0)
    }

    private var testView: some View {
        ApiAccessMethodTestView(
            status: viewModel.testState
        ) {
            viewModel.startTesting()
        } onStop: {
            viewModel.stopTesting()
        }
    }

    private var deleteButton: some View {
        MullvadButton(
            text: "Delete method", style: .destructive,
            action: {
                deleteConfirmationAlert = methodDeletionAlert
            }
        )
        .padding(.vertical, 24.0)
    }

    private var aboutSheet: some View {
        NavigationStack {
            if let infoModalConfig = viewModel.infoModalConfig {
                AboutView(
                    header: infoModalConfig.header,
                    preamble: infoModalConfig.preamble,
                    paragraphs: infoModalConfig.body.map {
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
            } else {
                EmptyView()
            }
        }
        .tint(Color.mullvadTextPrimary)

    }

    private var methodDeletionAlert: MullvadAlert {
        MullvadAlert(
            type: .info, messages: ["Delete **\(viewModel.name)**?"],
            actions: [
                .init(
                    type: .destructive, title: "Delete",
                    identifier: AccessibilityIdentifier.accessMethodConfirmDeleteButton,
                    handler: {
                        deleteConfirmationAlert = nil
                        viewModel.delete()
                        dismiss()
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
}
