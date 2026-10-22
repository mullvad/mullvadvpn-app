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

struct AddCustomListView: View {
    let customList: CustomListLocationNode?
    @State var availableLocations: CustomListSelectableLocationNode
    @State private var selectionHasChanged = false
    private var hasUnsavedChanges: Bool {
        selectionHasChanged || (customList?.name ?? "" != customListName)
    }

    let viewModel: any SelectLocationViewModel
    @State private var customListName: String
    private var isNewList: Bool {
        customList == nil
    }
    @FocusState private var isTextFieldFocused: Bool
    @Environment(\.dismissSheet) private var dismissSheet
    @Environment(\.dismiss) private var dismiss
    private let isRootView: Bool
    private let itemFactory = SegmentedListItemFactory()
    @State private var alert: MullvadAlert?
    // Used to check if there is unsaved changes
    private let initiallyLocationSelection: [Bool]
    private var currentLocationSelection: [Bool] {
        availableLocations.flattened.compactMap {
            ($0 as? CustomListSelectableLocationNode)?.isCustomListSelected
        }
    }

    init(
        availableLocations: [LocationNode],
        customList: CustomListLocationNode?,
        viewModel: any SelectLocationViewModel,
        isRootView: Bool = false
    ) {
        self.viewModel = viewModel
        self.customList = customList
        self.customListName = customList?.name ?? ""
        self.isRootView = isRootView
        let availableLocations = Self.setupAvailableLocations(
            availableLocations,
            customList
        )
        self.availableLocations = availableLocations
        self.initiallyLocationSelection = availableLocations.flattened.compactMap {
            ($0 as? CustomListSelectableLocationNode)?.isCustomListSelected
        }
    }

    var body: some View {
        SettingsContainerView {
            SegmentedListItem(
                leading: {
                    itemFactory.leading(for: .generic(title: NSLocalizedString("Name", comment: "")))
                },
                trailing: {
                    itemFactory.trailing(
                        for: .input(
                            title: "",
                            placeholder: "Custom",
                            text: $customListName,
                        )
                    )
                    .focused($isTextFieldFocused)
                },
                onSelect: {
                    isTextFieldFocused = true
                }
            )
            SegmentedListItem(
                accessibilityIdentifier: .customListEditAddOrEditLocationCell,
                leadingAndTrailingDestination: {
                    CustomListLocationSelectionView(
                        availableLocations: availableLocations
                    )
                    .navigationTitle(customListName)
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
            if let customList {
                MullvadButton(
                    text: "Delete custom list",
                    style: .destructivePrimary,
                    mainAccessibilityIdentifier: .customListEditDeleteListCell
                ) {
                    alert = .deleteCustomListAlert(
                        customList: customList,
                        onDelete: {
                            viewModel.delete(customList: customList.customList)
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
        .onChange(of: currentLocationSelection) {
            selectionHasChanged = currentLocationSelection != initiallyLocationSelection
        }
        .padding()
        .navigationBarBackButtonHidden(true)
        .toolbar {
            saveOrCreateToolbarButton()
            ToolbarItem(placement: .navigation) {
                Button {
                    if hasUnsavedChanges {
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
            if isNewList {
                $0.navigationTitle("New custom list")
            } else {
                $0.navigationTitle(customListName)
            }
        }
        .navigationBarTitleDisplayMode(.inline)
        .interactiveDismissDisabled(hasUnsavedChanges)
        .onAppear {
            if isNewList {
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
        return ToolbarItem(placement: .confirmationAction) {
            let title: LocalizedStringKey = isNewList ? "Create" : "Save"
            Button(title) {
                let locations = availableLocations
                    .flattened
                    .compactMap { $0 as? CustomListSelectableLocationNode }
                    .filter {
                        $0.isCustomListSelected
                    }
                    .filter {
                        if let parent = $0.parent as? CustomListSelectableLocationNode {
                            !parent.isCustomListSelected
                        } else {
                            true
                        }
                    }
                    .flatMap { $0.locations }
                var customList =
                    customList?.customList
                    ?? .init(
                        name: customListName,
                        locations: []
                    )
                customList.name = customListName
                customList.locations = locations
                do {
                    try viewModel.save(list: customList)
                    dismiss()
                } catch is CustomRelayListError {

                } catch {

                }
            }
            .disabled(customListName.isEmpty)
            .foregroundStyle(Color.mullvadTextPrimary)
            .accessibilityIdentifier(.saveCreateCustomListButton)
        }
        .hideLiquidGlassEffect()
    }

    private static func setupAvailableLocations(
        _ availableLocations: [LocationNode],
        _ customList: CustomListLocationNode?
    ) -> CustomListSelectableLocationNode {
        let availableLocations = CustomListSelectableLocationNode(
            fromLocationNode: LocationNode(
                name: "",
                code: "",
                children: availableLocations
            ))
        if let customList {
            customList.forEachDescendant { selectedLocation in
                availableLocations
                    .flattened
                    .filter {
                        $0.code == selectedLocation.codeWithoutCustomList
                    }
                    .compactMap { $0 as? CustomListSelectableLocationNode }
                    .forEach {
                        $0.isCustomListSelected = true
                    }
            }
            availableLocations.forEachDescendant {
                guard let node = $0 as? CustomListSelectableLocationNode else {
                    return
                }

                if !node.isCustomListSelected {
                    let hasSelectedChild = node.flattened.contains {
                        ($0 as? CustomListSelectableLocationNode)?.isCustomListSelected ?? false
                    }
                    node.showsChildren = hasSelectedChild
                }
            }
        }
        return availableLocations
    }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                AddCustomListView(
                    availableLocations: MockSelectLocationViewModel().entryContext.locations,
                    customList: nil,
                    viewModel: MockSelectLocationViewModel()
                )
            }
        }
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                AddCustomListView(
                    availableLocations: MockSelectLocationViewModel().entryContext.locations,
                    customList: .init(
                        name: "MyList",
                        code: "my-list",
                        customList: .init(name: "", locations: [])
                    ),
                    viewModel: MockSelectLocationViewModel()
                )
            }
        }
}

fileprivate extension LocationNode {
    var codeWithoutCustomList: String {
        code.split(separator: "-").dropFirst().joined(separator: "-")
    }
}
