import MullvadTypes
import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel

    var showSearchField: Bool { !viewModel.showDAITAInfo || viewModel.multihopContext == .exit }

    var body: some View {
        VStack(spacing: 16) {
            if viewModel.isMultihopEnabled {
                SegmentedControl(
                    segments: MultihopContext.allCases,
                    selectedSegment: $viewModel.multihopContext
                )
                .padding(.horizontal)
            }
            if showSearchField {
                MullvadSecondaryTextField(
                    placeholder: "Search for locations or servers",
                    text: $viewModel.searchText
                )
                .padding(.horizontal)
                .animation(.default, value: showSearchField)
                .transition(.move(edge: .top).combined(with: .opacity))
            }
            switch viewModel.multihopContext {
            case .exit:
                ExitLocationView(
                    viewModel: viewModel,
                    context: $viewModel.exitContext
                )
                .transition(
                    .move(edge: .trailing).combined(with: .opacity)
                )
                .geometryGroup()
            case .entry:
                EntryLocationView(viewModel: viewModel)
                    .transition(
                        .move(edge: .leading).combined(with: .opacity)
                    )
                    .geometryGroup()
            }
        }
        .animation(.default, value: viewModel.multihopContext)
        .animation(.default, value: viewModel.isMultihopEnabled)
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
                        Button {
                            viewModel.toggleMultihop()
                        } label: {
                            var title: LocalizedStringKey {
                                viewModel.isMultihopEnabled ? "Disable multihop" : "Enable multihop"
                            }
                            HStack {
                                Image.mullvadIconMultihop
                                    .renderingMode(.template)
                                Text(title)
                            }
                            .foregroundStyle(Color.mullvadTextPrimary)
                        }
                        .accessibilityIdentifier(.toggleMultihopButton)
                    } label: {
                        Image(systemName: "ellipsis.circle.fill")
                            .foregroundStyle(Color.mullvadTextPrimary)
                            .accessibilityIdentifier(.selectLocationToolbarMenu)
                    }
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
