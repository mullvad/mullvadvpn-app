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

class MockSelectLocationViewModel: SelectLocationViewModel {
    func onFilterTapped(_ filter: SelectLocationFilter) {
        print("show filter: \(filter)")
    }

    func onFilterRemoved(_ filter: SelectLocationFilter) {
        print("remove filter: \(filter)")
    }

    var searchText: String = ""

    var activeFilter: [SelectLocationFilter] = [
        .daita,
        .obfuscation,
        .rented,
        .owned,
        .provider(12),
    ]

    var connectedRelayHostname: String?

    var selectedLocation: LocationNode?

    var showFilterView: (() -> Void)?

    var showEditCustomListView: (([LocationNode]) -> Void)?

    var showAddCustomListView: (([LocationNode]) -> Void)?

    var didFinish: (() -> Void)?

    func onSelectLocation(_ location: LocationNode) {
        print("Selected location: \(location.name)")
    }

    @Published var customLists: [LocationNode] = [
        LocationNode(name: "MyList1", code: "mylist1", children: [
            LocationNode(name: "Sweden", code: "se", children: [
                LocationNode(
                    name: "Stockholm",
                    code: "sth",
                ),
                LocationNode(name: "Gothenburg", code: "gto", children: [
                    LocationNode(name: "se-got-001", code: "se-got-001"),
                    LocationNode(name: "se-got-002", code: "se-got-002"),
                    LocationNode(name: "se-got-003", code: "se-got-003"),
                ]),
            ]),
            LocationNode(name: "Gothenburg", code: "gto", children: [
                LocationNode(name: "se-got-001", code: "se-got-001"),
                LocationNode(name: "se-got-002", code: "se-got-002"),
            ]),
            LocationNode(name: "se-got-003", code: "se-got-003"),
        ]),
        LocationNode(name: "MyList2", code: "mylist2", children: [
            LocationNode(name: "Germany", code: "de", children: [
                LocationNode(name: "Berlin", code: "ber", children: [
                    LocationNode(name: "de-ber-001", code: "de-ber-001"),
                    LocationNode(name: "de-ber-002", code: "de-ber-002"),
                    LocationNode(name: "de-ber-003", code: "de-ber-003"),
                ]),
                LocationNode(name: "Frankfurt", code: "fra", children: [
                    LocationNode(name: "de-fra-001", code: "de-fra-001"),
                    LocationNode(name: "de-fra-002", code: "de-fra-002"),
                    LocationNode(name: "de-fra-003", code: "de-fra-003"),
                ]),
            ]),
        ]),
        LocationNode(
            name: "Stockholm",
            code: "sth",
        ),
    ]

    @Published var allLocations: [LocationNode] = [
        LocationNode(name: "Sweden", code: "se", children: [
            LocationNode(
                name: "Stockholm",
                code: "sth",
                children: [
                    LocationNode(name: "se-sto-001", code: "se-sto-001"),
                    LocationNode(name: "se-sto-002", code: "se-sto-002"),
                    LocationNode(name: "se-sto-003", code: "se-sto-003"),
                ]
            ),
            LocationNode(name: "Gothenburg", code: "gto", children: [
                LocationNode(name: "se-got-001", code: "se-got-001"),
                LocationNode(name: "se-got-002", code: "se-got-002"),
                LocationNode(name: "se-got-003", code: "se-got-003"),
            ]),
        ], showsChildren: true),
        LocationNode(name: "Germany", code: "de", children: [
            LocationNode(name: "Berlin", code: "ber", children: [
                LocationNode(name: "de-ber-001", code: "de-ber-001"),
                LocationNode(name: "de-ber-002", code: "de-ber-002"),
                LocationNode(name: "de-ber-003", code: "de-ber-003"),
            ]),
            LocationNode(name: "Frankfurt", code: "fra", children: [
                LocationNode(name: "de-fra-001", code: "de-fra-001"),
                LocationNode(name: "de-fra-002", code: "de-fra-002"),
                LocationNode(name: "de-fra-003", code: "de-fra-003"),
            ]),
        ]),
        LocationNode(name: "France", code: "fr", children: [
            LocationNode(name: "Paris", code: "par", children: [
                LocationNode(name: "fr-par-001", code: "fr-par-001"),
                LocationNode(name: "fr-par-002", code: "fr-par-002"),
                LocationNode(name: "fr-par-003", code: "fr-par-003"),
            ]),
            LocationNode(name: "Lyon", code: "lyo", children: [
                LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
            ]),
        ]),
    ]

    func refreshCustomLists() {}
}

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let onSelect: (SelectLocationFilter) -> Void
    let onRemove: (SelectLocationFilter) -> Void
    @State private var maxItemHeight: CGFloat = 0
    var body: some View {
        HStack(alignment: .top) {
            Text("Filtered:")
                .font(.mullvadTiny)
            MullvadHFlow(activeFilter) { filter in
                Button {
                    onSelect(filter)
                } label: {
                    HStack {
                        Text(filter.title)
                            .font(.mullvadMiniSemiBold)
                            .foregroundStyle(Color.mullvadTextPrimary)
                        if filter.canBeRemoved {
                            Button {
                                onRemove(filter)
                            } label: {
                                Image.mullvadIconCross
                            }
                        }
                    }
                    .padding(8)
                    .sizeOfView { size in
                        maxItemHeight = max(maxItemHeight, size.height)
                    }
                    .frame(height: maxItemHeight)
                    .background {
                        RoundedRectangle(cornerRadius: 8)
                            .foregroundStyle(Color.MullvadButton.primary)
                    }
                }
            }
        }
    }
}

#Preview {
    Text("da")
        .sheet(isPresented: .constant(true)) {
            NavigationView {
                ScrollView {
                    ActiveFilterView(
                        activeFilter: [.daita, .owned, .rented, .provider(2)],
                        onSelect: { _ in
                        },
                        onRemove: { _ in }
                    )
                }
            }
        }
}
