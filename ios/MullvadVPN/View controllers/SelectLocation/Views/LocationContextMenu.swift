import MullvadTypes
import SwiftUI

extension ExitLocationView {

    @ViewBuilder
    func customListContextMenu(_ location: LocationNode) -> some View {
        VStack {
            let isList = location.userSelectedRelays.customListSelection?.isList ?? false
            if isList {
                Button("Edit") {
                    viewModel.showEditCustomList(name: location.name)
                }

                Button("Delete") {
                    alert = .init(
                        type: .warning,
                        messages: ["Do you want to delete the list **\(location.name)**?"],
                        action: .init(
                            type: .danger,
                            title: "Delete list",
                            identifier: nil,
                            handler: {
                                viewModel.deleteCustomList(name: location.name)
                                alert = nil
                            }
                        ),
                        dismissButtonTitle: "Cancel"
                    )
                }
            } else {
                if let customListNode = location.parent?.asCustomListNode {
                    Button("Remove") {
                        viewModel
                            .removeLocationFromCustomList(
                                location: location,
                                customListName: customListNode.name
                            )
                        UIImpactFeedbackGenerator(
                            style: .medium
                        )
                        .impactOccurred()
                    }
                } else {
                    // Only top level nodes can be removed from a custom list
                    EmptyView()
                }
            }
        }
    }

    @ViewBuilder
    func locationContextMenu(_ location: LocationNode) -> some View {
        Section("Add \(location.name) to list") {
            ForEach(
                context.customLists,
                id: \.code
            ) { customList in
                var isAlreadyInList: Bool {
                    var isAlreadyInList = false
                    customList.forEachDescendant {
                        if $0.locations == location.locations {
                            isAlreadyInList = true
                        }
                    }
                    return isAlreadyInList
                }
                Button(customList.name) {
                    viewModel
                        .addLocationToCustomList(
                            location: location,
                            customListName: customList.name
                        )
                    UIImpactFeedbackGenerator(
                        style: .medium
                    )
                    .impactOccurred()
                }
                .disabled(isAlreadyInList)
            }
            Button {
                newCustomListAlert = .init(
                    title: "Create new list",
                    placeholder: "List name",
                    action: .init(
                        type: .default,
                        title: "Create",
                        identifier: nil,
                        handler: { listName in
                            viewModel
                                .addLocationToCustomList(
                                    location: location,
                                    customListName: listName
                                )
                            newCustomListAlert = nil
                        }
                    ),
                    validate: { listName in
                        !listName.isEmpty && listName.count <= NameInputFormatter.maxLength
                    },
                    dismissButtonTitle: "Cancel"
                )
            } label: {
                Label("New list", systemImage: "plus")
            }
        }
    }

    @ViewBuilder
    func menuForRecentLocation(_ location: LocationNode) -> some View {
        // If this location’s selected node still belongs to an existing custom list,
        // show the custom-list context menu. Otherwise, show the default menu so the
        // user can assign the location to a new custom list (prevents dangling selections
        // if a custom list was deleted).
        if let customListSelection = location.userSelectedRelays.customListSelection,
            let customLists = context.customLists as? [CustomListLocationNode],
            customLists.contains(where: { $0.customList.id == customListSelection.listId })
        {
            customListContextMenu(location)
        } else {
            locationContextMenu(location)
        }
    }
}
