import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    @State var animatedFilters: [SelectLocationFilter]?
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let multihopContext = viewModel.multihopContext {
                    SegmentedControl(
                        segments: MultihopContext.allCases,
                        selectedSegment: .init(
                            get: {
                                multihopContext
                            },
                            set: { newContext in
                                viewModel.multihopContext = newContext
                            }
                        )
                    )
                }
                if let animatedFilters,
                    !animatedFilters.isEmpty
                {
                    ActiveFilterView(
                        activeFilter: viewModel.activeLocationContext.filter
                    ) { filter in
                        viewModel.onFilterTapped(filter)
                    } onRemove: { filter in
                        viewModel.onFilterRemoved(filter)
                    }
                }
                switch viewModel.multihopContext {
                case .none, .some(.exit):
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
                case .some(.entry):
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
            .onChange(of: viewModel.activeLocationContext.filter) { newValue in
                withAnimation {
                    animatedFilters = newValue
                }
            }
            .onAppear {
                animatedFilters = viewModel.activeLocationContext.filter
            }
            .padding()
        }
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
                    } label: {
                        Image(systemName: "ellipsis.circle.fill")
                            .foregroundStyle(Color.mullvadTextPrimary)
                    }
                }
            )
        }
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
        var body: some View {
            MullvadSecondaryTextField(
                placeholder: "Search for locations or servers...",
                text: $viewModel.searchText
            )
            HStack {
                ListHeader(title: "Custom lists")
                Button {
                    viewModel.showAddCustomListView(
                        locations: viewModel.activeLocationContext
                            .locations)
                } label: {
                    Image.mullvadIconAdd
                        .padding(12)
                }
                if !viewModel.activeLocationContext.customLists.isEmpty {
                    Button {
                        viewModel.showEditCustomListView(
                            locations: viewModel.activeLocationContext.locations
                        )
                    } label: {
                        Image.mullvadIconEdit
                            .padding(12)
                    }
                }
            }
            LocationsListView(
                locations: $viewModel.activeLocationContext.customLists,
                selectedLocation: viewModel.activeLocationContext.selectedLocation,
                connectedRelayHostname: viewModel.activeLocationContext.connectedRelayHostname
            ) { location in
                viewModel.activeLocationContext.selectLocation(location)
            } contextMenu: { location in
                VStack {
                    Button("Remove") {
                        print("Remove \(location.name)")
                    }
                    Button("Edit") {
                        print("Edit \(location.name)")
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
                .padding(.bottom, 16)
            ListHeader(title: "All locations")
            LocationsListView(
                locations: $viewModel.activeLocationContext.locations,
                selectedLocation: viewModel.activeLocationContext.selectedLocation,
                connectedRelayHostname: viewModel.activeLocationContext.connectedRelayHostname
            ) { location in
                viewModel.activeLocationContext.selectLocation(location)
            } contextMenu: { location in
                Section("Add country to list") {
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
                    Button("+ New list") {
                        print("Create new list with \(location.name)")
                    }
                }
            }
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
            Rectangle()
                .frame(height: 1)
                .foregroundStyle(Color.mullvadTextPrimary)
        }
    }
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
