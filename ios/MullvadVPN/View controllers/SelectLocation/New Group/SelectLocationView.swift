import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if !viewModel.activeFilter.isEmpty {
                    ActiveFilterView(
                        activeFilter: viewModel.activeFilter) { filter in
                            viewModel.onFilterTapped(filter)
                        } onRemove: { filter in
                            viewModel.onFilterRemoved(filter)
                        }
                }
                MullvadSecondaryTextField(
                    placeholder: "Search for locations or servers...",
                    text: $viewModel.searchText
                )
                HStack {
                    ListHeader(title: "Custom lists")
                    Button {
                        viewModel.showAddCustomListView?(viewModel.allLocations)
                    } label: {
                        Image.mullvadIconAdd
                            .padding(12)
                    }
                    if !viewModel.customLists.isEmpty {
                        Button {
                            viewModel.showEditCustomListView?(
                                viewModel.allLocations
                            )
                        } label: {
                            Image.mullvadIconEdit
                                .padding(12)
                        }
                    }
                }
                LocationsListView(
                    locations: $viewModel.customLists,
                    selectedLocation: viewModel.selectedLocation,
                    connectedRelayHostname: viewModel.connectedRelayHostname
                ) { location in
                    viewModel.onSelectLocation(location)
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

                let text: LocalizedStringKey = viewModel.customLists.isEmpty
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
                    locations: $viewModel.allLocations,
                    selectedLocation: viewModel.selectedLocation,
                    connectedRelayHostname: viewModel.connectedRelayHostname
                ) { location in
                    viewModel.onSelectLocation(location)
                } contextMenu: { location in
                    Section("Add country to list") {
                        ForEach(
                            viewModel.customLists,
                            id: \.code
                        ) { customList in
                            Button(customList.name) {
                                print("Add \(location.name) to \(customList.name)")
                            }
                        }
                        Button("+ New list") {
                            print("Create new list with \(location.name)")
                        }
                    }
                }
            }
            .animation(.default, value: viewModel.activeFilter)
            // iOS 18 has a bug where the button press does not get cancelled on drag. This is a hacky fix
            // https://developer.apple.com/forums/thread/763436?answerId=829089022#829089022
//            .modifier(FixScrollViewWithTappedButton())
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
                        viewModel.didFinish?()
                    }
                    .foregroundStyle(Color.mullvadTextPrimary)
                }
            )
            ToolbarItem(
                placement: .topBarLeading,
                content: {
                    Menu {
                        Button {
                            viewModel.showFilterView?()
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

// iOS 18 has a bug where the button press does not get cancelled on drag. This is a hacky fix
// https://developer.apple.com/forums/thread/763436?answerId=829089022#829089022
struct FixScrollViewWithTappedButton: ViewModifier {
    @State private var isScrolling = false
    @ViewBuilder public func body(content: Content) -> some View {
        if #available(iOS 18, *) {
            content
                .disabled(isScrolling)
                .simultaneousGesture(
                    DragGesture()
                        .onChanged { _ in
                            isScrolling = true
                        }
                        .onEnded { _ in
                            isScrolling = false
                        }
                )
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

enum SelectLocationFilter: Hashable {
    case daita
    case obfuscation
    case owned
    case rented
    case provider(Int)

    var canBeRemoved: Bool {
        switch self {
        case .daita, .obfuscation:
            return false
        case .provider, .owned, .rented:
            return true
        }
    }

    var title: LocalizedStringKey {
        switch self {
        case .daita:
            return "Setting: Daita"
        case .obfuscation:
            return "Setting: Obfuscation"
        case .owned:
            return "Owned"
        case .rented:
            return "Rented"
        case .provider(let count):
            return "Providers: \(count)"
        }
    }
}

@MainActor
protocol SelectLocationViewModel: ObservableObject {
    var allLocations: [LocationNode] { get set }
    var customLists: [LocationNode] { get set }
    var selectedLocation: LocationNode? { get }
    var connectedRelayHostname: String? { get }
    var activeFilter: [SelectLocationFilter] { get }
    var searchText: String { get set }
    var showFilterView: (() -> Void)? { get }
    var showEditCustomListView: (([LocationNode]) -> Void)? { get }
    var showAddCustomListView: (([LocationNode]) -> Void)? { get }
    var didFinish: (() -> Void)? { get }
    func onSelectLocation(_ location: LocationNode)
    func onFilterTapped(_ filter: SelectLocationFilter)
    func onFilterRemoved(_ filter: SelectLocationFilter)
    func refreshCustomLists()
}
