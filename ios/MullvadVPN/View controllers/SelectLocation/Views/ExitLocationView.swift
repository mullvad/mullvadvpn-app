import SwiftUI

struct ExitLocationView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @Binding var context: LocationContext
    @State var newCustomListAlert: MullvadInputAlert?
    @State var alert: MullvadAlert?
    @State private var scrollPosition: String?

    var isShowingCustomListsSection: Bool {
        viewModel.searchText.isEmpty
            || (!viewModel.searchText.isEmpty
                && !context.customLists
                    .filter {
                        !$0.isHiddenFromSearch
                    }.isEmpty)
    }
    var isShowingAllLocationsSection: Bool {
        !context.locations.filter({ !$0.isHiddenFromSearch }).isEmpty
    }

    var body: some View {
        // All items in the list are arranged in a flat hierarchy
        ScrollView {
            LazyVStack(spacing: 0) {
                Group {
                    if !context.filter.isEmpty {
                        ActiveFilterView(
                            activeFilter: context.filter
                        ) { filter in
                            viewModel.onFilterTapped(filter)
                        } onRemove: { filter in
                            viewModel.onFilterRemoved(filter)
                        }
                        .padding(.bottom, 16)
                    }
                    Group {
                        if isShowingCustomListsSection {
                            customListSection(isShowingHeader: isShowingAllLocationsSection)
                        }
                        if isShowingAllLocationsSection {
                            allLocationsSection(isShowingHeader: isShowingCustomListsSection)
                        }
                        if !isShowingCustomListsSection && !isShowingAllLocationsSection {
                            Text("No result for \"\(viewModel.searchText)\", please try a different search term.")
                                .font(.mullvadMiniSemiBold)
                                .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                                .padding(.vertical)
                        }
                    }
                    .padding(.horizontal, 16)
                }
                .zIndex(3)  // prevent wrong overlapping during animations
            }
        }
        .scrollPosition(id: $scrollPosition, anchor: .center)
        .accessibilityIdentifier(.selectLocationView)
        .task {
            guard viewModel.searchText.isEmpty else { return }
            let selectedLocation = (context.locations + context.customLists)
                .flatMap { $0.flattened + [$0] }
                .first { $0.isSelected }
            // prevent some cells from not being populated after the scroll
            try? await Task.sleep(for: .milliseconds(25))
            scrollPosition = selectedLocation?.code
        }
        .mullvadInputAlert(item: $newCustomListAlert)
        .mullvadAlert(item: $alert)
    }

    @ViewBuilder
    func allLocationsSection(isShowingHeader: Bool) -> some View {
        //        VStack(alignment: .leading, spacing: 4) {
        if isShowingHeader {
            MullvadListSectionHeader(title: "All locations")
        }
        LocationsListView(
            locations: $context.locations,
            multihopContext: viewModel.multihopContext,
        ) { location in
            context.selectLocation(location)
        } contextMenu: { location in
            locationContextMenu(location)
        }
    }

    @ViewBuilder
    func customListSection(isShowingHeader: Bool) -> some View {
        //        VStack(alignment: .leading, spacing: 4) {
        if isShowingHeader {
            HStack(spacing: 0) {
                MullvadListSectionHeader(title: "Custom lists")
                Button {
                    viewModel.showAddCustomListView(
                        locations: context
                            .locations)
                } label: {
                    Image.mullvadIconAdd
                        .padding(.horizontal, 10)
                }
                .accessibilityIdentifier(.addNewCustomListButton)
                if !context.customLists.isEmpty {
                    Button {
                        viewModel.showEditCustomListView(
                            locations: context.locations
                        )
                    } label: {
                        Image.mullvadIconEdit
                            .padding(.horizontal, 10)
                    }
                    .accessibilityIdentifier(.editCustomListButton)
                }
            }
        }
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
            To create a custom list press the “+” or long press on a country, city, or server.
            """
            : """
            To add locations to a list, press the pen or long press on a country, city, or server.
            """
        Text(text)
            .font(.mullvadMini)
            .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
            .padding(.horizontal, context.customLists.isEmpty ? 0 : 16)
            .padding(.top, context.customLists.isEmpty ? 0 : 4)
            .padding(.bottom, 24)
    }
}

#Preview {
    @Previewable @State var viewModel = MockSelectLocationViewModel()
    ExitLocationView(
        viewModel: viewModel,
        context: $viewModel.exitContext,
        newCustomListAlert: nil,
        alert: nil
    )
    .background(Color.mullvadBackground)
}

#Preview("Empty lists") {
    @Previewable @State var viewModel = MockSelectLocationViewModel()
    ExitLocationView(
        viewModel: viewModel,
        context: $viewModel.entryContext,
        newCustomListAlert: nil,
        alert: nil
    )
    .background(Color.mullvadBackground)
}
