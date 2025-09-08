import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    ListHeader(title: "Custom lists")
                    Button {
                        viewModel.showAddCustomListView?([])
                    } label: {
                        Image.mullvadIconAdd
                            .padding(12)
                    }
                    if !viewModel.customLists.isEmpty {
                        Button {
                            viewModel.showEditCustomListView?(viewModel.customLists)
                        } label: {
                            Image.mullvadIconEdit
                                .padding(12)
                        }
                    }
                }
                LocationsListView(
                    locations: viewModel.customLists,
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
                    locations: viewModel.allLocations,
                    selectedLocation: viewModel.selectedLocation,
                    connectedRelayHostname: viewModel.connectedRelayHostname
                ) { location in
                    viewModel.onSelectLocation(location)
                }
            }
            .padding()
        }
        .background(Color.mullvadBackground)
        .navigationTitle("Select location")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(
                placement: .cancellationAction,
                content: {
                    Button {
                        viewModel.didFinish?()
                    } label: {
                        Image.mullvadIconCross
                    }
                }
            )
            ToolbarItem(
                placement: .topBarTrailing,
                content: {
                    Image.mullvadIconSearch
                }
            )
            ToolbarItem(
                placement: .topBarTrailing,
                content: {
                    Button {
                        viewModel.showFilterView?()
                    } label: {
                        Image(systemName: "line.3.horizontal.decrease")
                            .foregroundStyle(Color.gray)
                    }
                }
            )
        }
    }
}

#Preview {
    NavigationView {
        SelectLocationView(
            viewModel: MockSelectLocationViewModel()
        )
    }
}

@MainActor
protocol SelectLocationViewModel: ObservableObject {
    var allLocations: [LocationNode] { get }
    var customLists: [LocationNode] { get }
    var selectedLocation: LocationNode? { get }
    var connectedRelayHostname: String? { get }
    var showFilterView: (() -> Void)? { get }
    var showEditCustomListView: (([LocationNode]) -> Void)? { get }
    var showAddCustomListView: (([LocationNode]) -> Void)? { get }
    var didFinish: (() -> Void)? { get }
    func onSelectLocation(_ location: LocationNode)
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
