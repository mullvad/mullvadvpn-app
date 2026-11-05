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

@MainActor
protocol CustomListViewModel: Observable, AnyObject {
    var customList: CustomListLocationNode? { get }
    var availableLocations: CustomListSelectableLocationNode { set get }
    var selectionHasChanged: Bool { get }
    var customListName: String { set get }
    var error: CustomRelayListError? { get }

    func delete() async
    func save() async -> Bool
}

extension CustomListViewModel {
    var hasUnsavedChanges: Bool {
        selectionHasChanged || (customList?.name ?? "" != customListName)
    }

    var isNewList: Bool {
        customList == nil
    }
}

@Observable
class CustomListViewModelImpl: CustomListViewModel {
    var customList: CustomListLocationNode?

    var availableLocations: CustomListSelectableLocationNode {
        didSet {
            let currentLocationSelection = availableLocations.flattened.compactMap {
                ($0.asCustomListSelectableLocationNode)?.isCustomListSelected
            }
            selectionHasChanged = currentLocationSelection != initiallyLocationSelection
        }
    }

    var selectionHasChanged: Bool = false

    var customListName: String

    var error: CustomRelayListError? = nil

    // Used to check if there is unsaved changes
    private let initiallyLocationSelection: [Bool]

    private var onSave: ((CustomList) async throws -> Void)?
    private var onDelete: ((CustomList) async -> Void)?

    init(
        customList: CustomListLocationNode?,
        availableLocations: [LocationNode],
        onSave: ((CustomList) async throws -> Void)? = nil,
        onDelete: ((CustomList) async -> Void)? = nil
    ) {
        self.customList = customList
        self.customListName = customList?.name ?? ""

        let availableLocations = Self.setupAvailableLocations(
            availableLocations,
            customList
        )
        self.availableLocations = availableLocations
        self.initiallyLocationSelection = availableLocations.flattened.compactMap {
            ($0.asCustomListSelectableLocationNode)?.isCustomListSelected
        }
        self.onSave = onSave
        self.onDelete = onDelete
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
                    .compactMap { $0.asCustomListSelectableLocationNode }
                    .forEach {
                        $0.isCustomListSelected = true
                    }
            }
            availableLocations.forEachDescendant {
                guard let node = $0.asCustomListSelectableLocationNode else {
                    return
                }

                if !node.isCustomListSelected {
                    let hasSelectedChild = node.flattened.contains {
                        ($0.asCustomListSelectableLocationNode)?.isCustomListSelected ?? false
                    }
                    node.showsChildren = hasSelectedChild
                }
            }
        }
        return availableLocations
    }

    func delete() async {
        guard let customList = customList?.customList else { return }
        await onDelete?(customList)
    }

    func save() async -> Bool {
        let locations = availableLocations
            .flattened
            .compactMap { $0.asCustomListSelectableLocationNode }
            .filter {
                $0.isCustomListSelected
            }
            .filter {
                if let parent = $0.parent?.asCustomListSelectableLocationNode {
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
                locations: locations
            )
        customList.name = customListName
        customList.locations = locations
        do {
            try await onSave?(customList)
            return true
        } catch is CustomRelayListError {
            self.error = error
        } catch {
            // This should never happen
        }
        return false
    }
}

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
                    .navigationTitle(viewModel.customListName)
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
                        customList: customList,
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
            let title: LocalizedStringKey = viewModel.isNewList ? "Create" : "Save"
            Button(title) {
                Task { @MainActor in
                    if await viewModel.save() {
                        dismiss()
                    }
                }
            }
            .disabled(viewModel.customListName.isEmpty)
            .foregroundStyle(Color.mullvadTextPrimary)
            .accessibilityIdentifier(.saveCreateCustomListButton)
        }
        .hideLiquidGlassEffect()
    }
}

class MockCustomListViewModel: CustomListViewModel {
    let customList: CustomListLocationNode? = nil

    lazy var availableLocations: CustomListSelectableLocationNode = {
        CustomListSelectableLocationNode(fromLocationNode: RootLocationNode(children:
        MockSelectLocationViewModel().entryContext.locations))
    }()

    let selectionHasChanged: Bool = false

    var customListName: String = "MyList"

    let error: CustomRelayListError?

    init(error: CustomRelayListError?) {
        self.error = error
    }

    func delete() async {

    }

    func save() async -> Bool {
        return false
    }


}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                CustomListView(
                    viewModel: CustomListViewModelImpl(
                        customList: nil,
                        availableLocations: MockSelectLocationViewModel().entryContext.locations
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
                    viewModel: CustomListViewModelImpl(
                        customList: .init(
                            name: "MyList",
                            code: "my-list",
                            customList: .init(name: "", locations: [])
                        ),
                        availableLocations: MockSelectLocationViewModel().entryContext.locations
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

fileprivate extension LocationNode {
    var codeWithoutCustomList: String {
        code.split(separator: "-").dropFirst().joined(separator: "-")
    }
}
