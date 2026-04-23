import MullvadTypes
import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    @State private var headerIsExpandedForEntry: Bool = false
    @State private var headerIsExpandedForExit: Bool = false
    @State private var disablingRecentConnectionsAlert: MullvadAlert?
    @FocusState private var focusSearchField: Bool
    @State private var isSearchExpanded: Bool = false
    @State private var headerHeight: CGFloat = 0
    @State private var floatingBarHeight: CGFloat = 0
    @ScaledMetric(relativeTo: .body) private var listBottomInset: CGFloat = 56

    private var headerIsExpanded: Bool {
        switch viewModel.multihopContext {
        case .entry:
            headerIsExpandedForEntry
        case .exit:
            headerIsExpandedForExit
        }
    }

    private var showSearchField: Bool {
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
                            let selectedLocation =
                                switch $0 {
                                case .entry:
                                    viewModel.showDAITAInfo
                                        ? LocationNode(name: "Automatic", code: "")
                                        : viewModel.entryContext.selectedLocation
                                case .exit: viewModel.exitContext.selectedLocation
                                }
                            return Hop(
                                multihopContext: $0,
                                selectedLocation: selectedLocation
                            )
                        },
                    selectedMultihopContext: $viewModel.multihopContext,
                    isExpanded: headerIsExpanded
                )
                .padding(.horizontal, 16)
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
                            onScrollVisibilityChange: {
                                expandHeader in
                                withAnimation {
                                    headerIsExpandedForExit = expandHeader
                                }
                            }
                        )
                        .transition(
                            .move(edge: .trailing).combined(with: .opacity)
                        )
                    case .entry:
                        EntryLocationView(
                            viewModel: viewModel,
                            onScrollVisibilityChange: {
                                expandHeader in
                                withAnimation {
                                    headerIsExpandedForEntry = expandHeader
                                }
                            }
                        )
                        .transition(
                            .move(edge: .leading).combined(with: .opacity)
                        )
                    }
                }
                .simultaneousGesture(
                    DragGesture(minimumDistance: 50)
                        .onChanged { _ in
                            focusSearchField = false
                        }
                )
                .environment(\.dismissSearchFocus, { focusSearchField = false })
                .geometryGroup()
                // Adds margin to the top of the scroll content. The scroll views size stays untouched
                // which seems to be the solution to animation issues.
                .contentMargins(.top, headerHeight - 1)
                .contentMargins(.bottom, showSearchField ? floatingBarHeight + listBottomInset : 0)
                .zIndex(0)
            }
        }
        .overlay(alignment: .bottom) {
            FloatingSearchBar(
                searchText: $viewModel.searchText,
                isExpanded: $isSearchExpanded,
                isFocused: $focusSearchField
            )
            .showIf(showSearchField)
            .padding(.horizontal, 16)
            .padding(.bottom, 16)
            .sizeOfView { floatingBarHeight = $0.height }
            .accessibilitySortPriority(1)
        }
        .onChange(of: showSearchField) { _, newValue in
            if !newValue {
                isSearchExpanded = false
                viewModel.searchText = ""
            }
        }
        .animation(.default, value: isSearchExpanded)
        .animation(.default, value: showSearchField)
        .animation(.default, value: viewModel.multihopContext)
        .animation(.default, value: viewModel.isMultihopEnabled)
        .animation(.default, value: viewModel.isRecentsEnabled)
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
                                Text("Filters")
                                Image.mullvadIconFilter
                                    .renderingMode(.template)
                            }
                        }
                        .accessibilityIdentifier(.selectLocationFilterButton)

                        Button {
                            viewModel.toggleMultihop()
                        } label: {
                            var title: LocalizedStringKey {
                                viewModel.isMultihopEnabled ? "Disable multihop" : "Enable multihop"
                            }
                            HStack {
                                Text(title)
                                viewModel.isMultihopEnabled
                                    ? Image.mullvadIconDisableMultihop
                                        .renderingMode(.template)
                                    : Image.mullvadIconEnableMultihop
                                        .renderingMode(.template)
                            }
                        }
                        .accessibilityIdentifier(.toggleMultihopButton)

                        Button {
                            if viewModel.isRecentsEnabled {
                                disablingRecentConnectionsAlert = MullvadAlert(
                                    type: .warning,
                                    messages: ["Disabling recents will also clear history."],
                                    actions: [
                                        MullvadAlert.Action(
                                            type: .danger,
                                            title: "Disable",
                                            identifier: AccessibilityIdentifier.disableRecentConnectionsButton,
                                            handler: {
                                                disablingRecentConnectionsAlert = nil
                                                viewModel.toggleRecents()
                                            }
                                        ),
                                        MullvadAlert.Action(
                                            type: .default,
                                            title: "Cancel",
                                            handler: {
                                                disablingRecentConnectionsAlert = nil
                                            }
                                        ),
                                    ]
                                )

                            } else {
                                viewModel.toggleRecents()
                            }

                        } label: {
                            HStack {
                                Text(viewModel.isRecentsEnabled ? "Disable recents" : "Enable recents")
                                viewModel.isRecentsEnabled
                                    ? Image.mullvadIconDisableRecents
                                        .renderingMode(.template)
                                    : Image.mullvadIconEnableRecents
                                        .renderingMode(.template)
                            }
                        }
                        .accessibilityIdentifier(.recentConnectionsToggleButton)

                        Button {
                            viewModel.manuallyFetchRelayList()
                        } label: {
                            HStack {
                                Text("Update server list")
                                Image(systemName: "arrow.clockwise")
                            }
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle.fill")
                            .foregroundStyle(Color.mullvadTextPrimary)
                            .accessibilityIdentifier(.selectLocationToolbarMenu)
                    }
                }
            )
        }
        .mullvadAlert(item: $disablingRecentConnectionsAlert)
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
