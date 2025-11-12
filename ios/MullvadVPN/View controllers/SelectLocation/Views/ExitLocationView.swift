import SwiftUI

struct ExitLocationView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @Binding var context: LocationContext
    @State var newCustomListAlert: MullvadInputAlert?
    @State var alert: MullvadAlert?
    var isShowingCustomListsSection: Bool {
        viewModel.searchText.isEmpty
            || (!viewModel.searchText.isEmpty
                && !context.customLists
                    .filter {
                        !$0.isHiddenFromSearch
                    }.isEmpty)
    }

    @State private var topOfTheListId = UUID()

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading) {
                    EmptyView()
                        .id(topOfTheListId)
                    if !context.filter.isEmpty {
                        ActiveFilterView(
                            activeFilter: context.filter
                        ) { filter in
                            viewModel.onFilterTapped(filter)
                        } onRemove: { filter in
                            viewModel.onFilterRemoved(filter)
                        }
                    }
                    if isShowingCustomListsSection {
                        HStack {
                            MullvadListSectionHeader(title: "Custom lists")
                            Button {
                                viewModel.showAddCustomListView(
                                    locations: context
                                        .locations)
                            } label: {
                                Image.mullvadIconAdd
                                    .padding(.horizontal, 12)
                            }
                            .accessibilityIdentifier(.addNewCustomListButton)
                            if !context.customLists.isEmpty {
                                Button {
                                    viewModel.showEditCustomListView(
                                        locations: context.locations
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
                            locations: $context.customLists,
                            multihopContext: viewModel.multihopContext,
                        ) { location in
                            context.selectLocation(location)
                        } contextMenu: { location in
                            customListContextMenu(location)
                        }

                        let text: LocalizedStringKey =
                            context.customLists.isEmpty
                            ? """
                            Save locations by adding them to a custom list.
                            """
                            : """
                            To add locations to a list, press the pen or long press on a country, city, or server.
                            """
                        Text(text)
                            .font(.mullvadMini)
                            .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                        MullvadListSectionHeader(title: "All locations")
                            .padding(.vertical, 12)
                    }
                    if !viewModel.searchText.isEmpty
                        && context.locations
                            .filter({ !$0.isHiddenFromSearch }).isEmpty
                    {
                        Text("No result for \"\(viewModel.searchText)\", please try a different search term.")
                            .font(.mullvadMiniSemiBold)
                            .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                            .padding(.vertical)
                    } else {
                        LocationsListView(
                            locations: $context.locations,
                            multihopContext: viewModel.multihopContext,
                        ) { location in
                            context.selectLocation(location)
                        } contextMenu: { location in
                            locationContextMenu(location)
                        }
                    }
                }
                .transformEffect(.identity)
                .padding(.horizontal)
                .padding(.bottom)
            }
            .onChange(of: viewModel.searchText) {
                proxy.scrollTo(topOfTheListId, anchor: .top)
            }
            .onAppear {
                guard viewModel.searchText.isEmpty else { return }
                let selectedLocation = (context.locations + context.customLists)
                    .flatMap { $0.flattened + [$0] }
                    .first { $0.isSelected }
                if let selectedLocation {
                    var rootParent = selectedLocation
                    while let parent = rootParent.parent {
                        rootParent = parent
                    }
                    Task {
                        // Due to the use of LazyVStack the view can not scroll to child nodes that are outside the viewport
                        // Therefor the view must scroll to the root parent first
                        if rootParent != selectedLocation {
                            proxy.scrollTo(rootParent.code, anchor: .center)
                        }
                        try? await Task.sleep(for: .milliseconds(50))
                        proxy.scrollTo(selectedLocation.code, anchor: .center)
                    }
                }
            }
        }
        .animation(.default, value: context.filter)
        .mullvadInputAlert(item: $newCustomListAlert)
        .mullvadAlert(item: $alert)
    }
}
