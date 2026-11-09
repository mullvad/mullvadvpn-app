// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct CustomListView<ViewModel: CustomListViewModel>: View {
    @State private var viewModel: ViewModel
    private let isRootView: Bool
    private let itemFactory: SegmentedListItemFactory

    @State private var alert: MullvadAlert?

    @FocusState private var isTextFieldFocused: Bool
    @Environment(\.dismissSheet) private var dismissSheet
    @Environment(\.dismiss) private var dismiss

    init(
        viewModel: ViewModel,
        isRootView: Bool = false
    ) {
        self.viewModel = viewModel
        self.isRootView = isRootView

        self.itemFactory = SegmentedListItemFactory()
    }

    var body: some View {
        SettingsContainerView {
            VStack(spacing: 2) {
                SegmentedListItem(
                    appearence: viewModel.error != nil ? .error : .normal,
                    leading: {
                        itemFactory
                            .leading(
                                for: .generic(
                                    title: NSLocalizedString("Name", comment: "")
                                )
                            )
                    },
                    trailing: {
                        itemFactory.trailing(
                            for: .input(
                                title: "",
                                placeholder: "Custom list",
                                text: $viewModel.customListName,
                            )
                        )
                        .focused($isTextFieldFocused)
                    },
                    footer: {
                        if let error = viewModel.error {
                            itemFactory
                                .footer(
                                    for: .generic(
                                        text: error.localizedDescription
                                    )
                                )
                        }
                    },
                    onSelect: {
                        isTextFieldFocused = true
                    }
                )
            }
            SegmentedListItem(
                accessibilityIdentifier: .customListEditAddOrEditLocationCell,
                leadingAndTrailingDestination: {
                    CustomListLocationSelectionView(
                        viewModel: viewModel
                    )
                    .navigationTitle("Locations")
                    .toolbar {
                        saveOrCreateToolbarButton()
                    }
                },
                leading: {
                    itemFactory
                        .leading(
                            for: .generic(title: NSLocalizedString("Locations", comment: ""))
                        )
                },
                trailing: {
                    itemFactory.trailing(
                        for: .drillDown(title: "", breadcrumb: nil)
                    )
                }
            )
            if let customList = viewModel.customList {
                MullvadButton(
                    text: "Delete custom list",
                    style: .destructivePrimary,
                    mainAccessibilityIdentifier: .customListEditDeleteListCell
                ) {
                    alert = .deleteCustomListAlert(
                        customListName: customList.name,
                        onDelete: {
                            await viewModel.delete()
                            alert = nil
                            dismissSheet()
                        },
                        onCancel: {
                            alert = nil
                        }
                    )
                }
            }
        }
        .padding()
        .navigationBarBackButtonHidden(true)
        .toolbar {
            saveOrCreateToolbarButton()
            ToolbarItem(placement: .navigation) {
                Button {
                    if viewModel.hasUnsavedChanges {
                        showUnsavedChangesAlert {
                            dismiss()
                        }
                    } else {
                        dismiss()
                    }
                } label: {
                    if isRootView {
                        Text("Cancel")
                    } else {
                        Image(systemName: "chevron.left")
                    }
                }
                .foregroundStyle(Color.mullvadTextPrimary)
            }
            .hideLiquidGlassEffect()
        }
        .apply {
            if viewModel.isNewList {
                $0.navigationTitle("New custom list")
            } else {
                $0.navigationTitle(viewModel.customListName)
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .interactiveDismissDisabled(viewModel.hasUnsavedChanges)
        .onAppear {
            if viewModel.isNewList {
                isTextFieldFocused = true
            }
        }
        .background(Color.mullvadBackground.ignoresSafeArea())
        .mullvadAlert(item: $alert)
        .accessibilityIdentifier(.newCustomListView)
    }

    private func showUnsavedChangesAlert(_ onDiscard: @escaping () -> Void) {
        isTextFieldFocused = false
        alert = .init(
            type: .warning,
            messages: ["You have unsaved changes."],
            actions: [
                .init(
                    type: .destructive, title: "Discard changes",
                    handler: {
                        alert = nil
                        onDiscard()
                    }),
                .init(
                    type: .primary, title: "Cancel",
                    handler: {
                        alert = nil
                    }),
            ]
        )
    }

    private func saveOrCreateToolbarButton() -> some ToolbarContent {
        var isDisabled: Bool {
            viewModel.customListName.isEmpty || viewModel.error != nil
        }
        return ToolbarItem(placement: .confirmationAction) {
            let title: LocalizedStringKey = viewModel.isNewList ? "Create" : "Save"
            Button(title) {
                Task { @MainActor in
                    if await viewModel.save() {
                        dismiss()
                    }
                }
            }
            .disabled(isDisabled)
            .foregroundStyle(
                isDisabled ? Color.mullvadTextPrimaryDisabled : Color.mullvadTextPrimary
            )
            .accessibilityIdentifier(.saveCreateCustomListButton)
        }
        .hideLiquidGlassEffect()
    }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                CustomListView(
                    viewModel: MockCustomListViewModel()
                )
            }
        }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                CustomListView(
                    viewModel: MockCustomListViewModel(
                        customList: .init(name: "myList", locations: [])
                    )
                )
            }
        }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                CustomListView(
                    viewModel: MockCustomListViewModel(error: .nameTooLong)
                )
            }
        }
}
