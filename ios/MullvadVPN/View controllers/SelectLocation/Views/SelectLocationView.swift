import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    //    @State var animatedFilters: [SelectLocationFilter]?
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if viewModel.isMultihopEnabled {
                    SegmentedControl(
                        segments: MultihopContext.allCases,
                        selectedSegment: $viewModel.multihopContext
                    )
                }
                switch viewModel.multihopContext {
                case .exit:
                    ExitLocationView(viewModel: viewModel)
                        .transition(
                            .move(edge: .trailing).combined(with: .opacity)
                        )
                        .apply {
                            if #available(iOS 17.0, *) {
                                $0.geometryGroup()
                            } else {
                                $0
                            }
                        }
                case .entry:
                    EntryLocationView(viewModel: viewModel)
                        .transition(
                            .move(edge: .leading).combined(with: .opacity)
                        )
                        .apply {
                            if #available(iOS 17.0, *) {
                                $0.geometryGroup()
                            } else {
                                $0
                            }
                        }
                }
            }
            .transformEffect(.identity)
            .padding()
        }
        .animation(.default, value: viewModel.multihopContext)
        .background(Color.mullvadBackground)
        .navigationTitle("Select location")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(
                placement: .topBarTrailing,
                content: {
                    Button("Done") {
                        viewModel.didFinish()
                    }
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .accessibilityIdentifier(.closeSelectLocationButton)
                }
            )
            ToolbarItem(
                placement: .topBarLeading,
                content: {
                    Menu {
                        Button {
                            viewModel.showFilterView()
                        } label: {
                            HStack {
                                Image(systemName: "line.3.horizontal.decrease")
                                Text("Filters")
                            }
                            .foregroundStyle(Color.mullvadTextPrimary)
                        }
                        .accessibilityIdentifier(.selectLocationFilterButton)
                    } label: {
                        Image(systemName: "ellipsis.circle.fill")
                            .foregroundStyle(Color.mullvadTextPrimary)
                            .accessibilityIdentifier(.selectLocationToolbarMenu)
                    }
                }
            )
        }
        .accessibilityIdentifier(.selectLocationView)
    }

    struct EntryLocationView: View {
        @ObservedObject var viewModel: ViewModel

        var body: some View {
            if viewModel.showDAITAInfo {
                DaitaWarningView {
                    viewModel.showDaitaSettings()
                }
            } else {
                ExitLocationView(viewModel: viewModel)
            }
        }
    }

    struct ExitLocationView: View {
        @ObservedObject var viewModel: ViewModel
        @State var newCustomListAlert: MullvadInputAlert?
        @State var alert: MullvadAlert?
        var body: some View {
            VStack {
                if !viewModel.activeLocationContext.filter.isEmpty {
                    ActiveFilterView(
                        activeFilter: viewModel.activeLocationContext.filter
                    ) { filter in
                        viewModel.onFilterTapped(filter)
                    } onRemove: { filter in
                        viewModel.onFilterRemoved(filter)
                    }
                }
                MullvadSecondaryTextField(
                    placeholder: "Search for locations or servers...",
                    text: $viewModel.searchText
                )
                if viewModel.searchText.isEmpty
                    || (!viewModel.searchText.isEmpty
                        && !viewModel.activeLocationContext.customLists
                            .filter {
                                !$0.isHiddenFromSearch
                            }.isEmpty)
                {
                    HStack {
                        ListHeader(title: "Custom lists")
                        Button {
                            viewModel.showAddCustomListView(
                                locations: viewModel.activeLocationContext
                                    .locations)
                        } label: {
                            Image.mullvadIconAdd
                                .padding(.horizontal, 12)
                        }
                        .accessibilityIdentifier(.addNewCustomListButton)
                        if !viewModel.activeLocationContext.customLists.isEmpty {
                            Button {
                                viewModel.showEditCustomListView(
                                    locations: viewModel.activeLocationContext.locations
                                )
                            } label: {
                                Image.mullvadIconEdit
                                    .padding(.horizontal, 12)
                            }
                            .accessibilityIdentifier(.editCustomListButton)
                        }
                    }
                    .padding(.vertical, 12)
                    LocationsListView(
                        locations: $viewModel.activeLocationContext.customLists,
                        multihopContext: viewModel.multihopContext,
                    ) { location in
                        viewModel.activeLocationContext.selectLocation(location)
                    } contextMenu: { location in
                        VStack {
                            switch location {
                            case let location as CustomListLocationNode:
                                Button("Edit") {
                                    viewModel.editCustomList(name: location.name)
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

                            default:
                                if let customListNode = location.parent as? CustomListLocationNode {
                                    Button("Remove") {
                                        viewModel
                                            .removeLocationFromCustomList(
                                                location: location,
                                                customListName: customListNode.name
                                            )
                                    }
                                } else {
                                    // Only top level nodes can be removed from a custom list
                                    EmptyView()
                                }
                            }
                        }
                    }

                    let text: LocalizedStringKey =
                        viewModel.activeLocationContext.customLists.isEmpty
                        ? """
                        Save locations by adding them to a custom list.
                        """
                        : """
                        To add locations to a list, press the pen or long press on a country, city, or server.
                        """
                    Text(text)
                        .font(.mullvadMini)
                        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                    ListHeader(title: "All locations")
                        .padding(.vertical, 12)
                }
                if !viewModel.searchText.isEmpty
                    && viewModel.activeLocationContext.locations
                        .filter({ !$0.isHiddenFromSearch }).isEmpty
                {
                    Text("No result for \"\(viewModel.searchText)\", please try with a different search term.")
                        .font(.mullvadMiniSemiBold)
                        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                        .padding(.vertical)
                } else {
                    LocationsListView(
                        locations: $viewModel.activeLocationContext.locations,
                        multihopContext: viewModel.multihopContext,
                    ) { location in
                        viewModel.activeLocationContext.selectLocation(location)
                    } contextMenu: { location in
                        Section("Add \(location.name) to list") {
                            ForEach(
                                viewModel.activeLocationContext.customLists,
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
                                }
                                .disabled(isAlreadyInList)
                            }
                            Button {
                                newCustomListAlert = .init(
                                    title: "Add new list",
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
                                        !listName.isEmpty && listName.count <= 32
                                    },
                                    dismissButtonTitle: "Cancel"
                                )
                            } label: {
                                Label("New list", systemImage: "plus")
                            }
                        }
                    }
                }
            }
            .animation(.default, value: viewModel.activeLocationContext.filter)
            .mullvadInputAlert(item: $newCustomListAlert)
            .mullvadAlert(item: $alert)
        }
    }
}

private struct ListHeader: View {
    let title: LocalizedStringKey

    var body: some View {
        HStack {
            Text(title)
                .font(.mullvadTiny)
                .foregroundStyle(Color.mullvadTextPrimary)
                .layoutPriority(1)
            Rectangle()
                .frame(height: 1)
                .foregroundStyle(Color.mullvadTextPrimary)
        }
        .frame(minHeight: 24, alignment: .center)
    }
}
#Preview {
    VStack {
        Spacer()
        HStack {
            ListHeader(title: "tatata")
            Text("dasdadsa")
                .foregroundStyle(Color.white)
        }
        Spacer()
    }
    .background(Color.black)
}

#Preview {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationView {
                SelectLocationView(
                    viewModel: MockSelectLocationViewModel()
                )
            }
        }
}
