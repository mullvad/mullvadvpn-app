import MullvadTypes
import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel

    @State private var isAtTopOfList: Bool = false
    @State private var headerHeight: CGFloat = 0

    var showSearchField: Bool {
        return !viewModel.showDAITAInfo || viewModel.multihopContext == .exit
    }

    var body: some View {
        // Simply animating the MultihopSelectionView while scrolling leads to a slow
        // down of the scrolling during the animation. Instead of changing the size of the scroll
        // view, the top margin of the content is changed which solves the animation issues.
        ZStack(alignment: .topLeading) {
            VStack(spacing: 16) {
                MultihopSelectionView(
                    hops: (viewModel.isMultihopEnabled ? MultihopContext.allCases : [MultihopContext.exit])
                        .map {
                            Hop(
                                multihopContext: $0,
                                selectedLocation: $0 == .entry
                                    ? viewModel.entryContext.selectedLocation : viewModel.exitContext.selectedLocation
                            )
                        },
                    selectedMultihopContext: $viewModel.multihopContext,
                    isExpanded: isAtTopOfList
                )
                .padding(.horizontal, 16)
                if showSearchField {
                    MullvadSecondaryTextField(
                        placeholder: "Search for locations or servers",
                        text: $viewModel.searchText
                    )
                    .padding(.horizontal)
                    .transition(.move(edge: .top).combined(with: .opacity))
                }
            }
            .padding(.vertical)
            .background(Color.mullvadDarkBackground)
            .zIndex(1)
            .sizeOfView { size in
                withAnimation {
                    headerHeight = size.height
                }
            }
            VStack {
                // Prevent scroll content from touching navigation bar to avoid a change of appearence
                // see `UINavigationBar+Appearance.swift`
                Spacer()
                    .frame(height: 1)
                Group {
                    switch viewModel.multihopContext {
                    case .exit:
                        ExitLocationView(
                            viewModel: viewModel,
                            context: $viewModel.exitContext,
                            onShowsTopOfTheListChange: { onTop in
                                withAnimation {
                                    isAtTopOfList = onTop
                                }
                            }
                        )
                        .transition(
                            .move(edge: .trailing).combined(with: .opacity)
                        )
                    case .entry:
                        EntryLocationView(
                            viewModel: viewModel,
                            onShowsTopOfTheListChange: { onTop in
                                withAnimation {
                                    isAtTopOfList = onTop
                                }
                            }
                        )
                        .transition(
                            .move(edge: .leading).combined(with: .opacity)
                        )
                    }
                }
                .geometryGroup()
                // Adds margin to the top of the scroll content. The scroll views size stays untouched
                // which seems to be the solution to animation issues.
                .contentMargins(.top, headerHeight - 1)
                .zIndex(0)
            }
        }
        .animation(.default, value: showSearchField)
        .animation(.default, value: viewModel.multihopContext)
        .background(Color.mullvadDarkBackground)
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
