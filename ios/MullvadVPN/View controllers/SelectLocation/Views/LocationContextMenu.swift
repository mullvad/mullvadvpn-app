import MullvadTypes
import SwiftUI

extension ExitLocationView {

    @ViewBuilder
    func customListContextMenu(_ location: LocationNode) -> some View {
        VStack {
            switch location {
            case let location as CustomListLocationNode:
                Section(LocalizedStringKey(location.name)) {
                    Button {
                        viewModel.showEditCustomList(name: location.name)
                    } label: {
                        HStack {
                            Text("Edit list")
                            Image.mullvadIconEdit
                                .renderingMode(.template)
                        }
                    }

                    Button(role: .destructive) {
                        alert = .init(
                            type: .warning,
                            messages: ["Do you want to delete the list **\(location.name)**?"],
                            actions: [
                                .init(
                                    type: .danger,
                                    title: "Delete list",
                                    handler: {
                                        viewModel.deleteCustomList(name: location.name)
                                        alert = nil
                                    }
                                ),
                                .init(
                                    type: .default,
                                    title: "Cancel",
                                    handler: {
                                        alert = nil
                                    }
                                ),
                            ]
                        )
                    } label: {
                        HStack {
                            Text("Delete list")
                            Image.mullvadIconDelete
                                .renderingMode(.template)
                        }
                    }
                }

            default:
                if let customListNode = location.parent?.asCustomListNode {
                    Button(role: .destructive) {
                        viewModel
                            .removeLocationFromCustomList(
                                location: location,
                                customListName: customListNode.name
                            )
                        UIImpactFeedbackGenerator(
                            style: .medium
                        )
                        .impactOccurred()
                    } label: {
                        HStack {
                            Text("Remove")
                            Image.mullvadIconDelete
                                .renderingMode(.template)
                        }
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
                HStack {
                    Text("New list")
                    Image.mullvadIconAdd
                        .renderingMode(.template)
                }
            }
        }
    }

    @ViewBuilder
    func recentLocationContextMenu(_ location: LocationNode) -> some View {
        if let customListNode = location.parent?.asCustomListNode,
            location.userSelectedRelays.customListSelection == customListNode.userSelectedRelays.customListSelection
        {
            customListContextMenu(customListNode)
        } else {
            locationContextMenu(location)
        }
    }
}
