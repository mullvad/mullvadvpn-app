import MullvadSettings
import MullvadTypes
import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    @State private var headerIsExpandedForEntry: Bool = false
    @State private var headerIsExpandedForExit: Bool = false
    @State private var disablingRecentConnectionsAlert: MullvadAlert?
    @State private var multihopBlockedStateWarningAlert: MullvadAlert?
    @FocusState private var focusSearchField: Bool
    @State private var isSearchExpanded: Bool = false
    @State private var headerHeight: CGFloat = 0
    @State private var floatingBarHeight: CGFloat = 0
    @ScaledMetric(relativeTo: .body) private var listBottomInset: CGFloat = 56

    private var headerIsExpanded: Bool {
        switch viewModel.multihopContext {
        case .entry:
            viewModel.showMultihopInfo || headerIsExpandedForEntry
        case .exit:
            headerIsExpandedForExit
        }
    }

    private var showSearchField: Bool {
        return !viewModel.showMultihopInfo || viewModel.multihopContext == .exit
    }

    var body: some View {
        // Simply animating the MultihopSelectionView while scrolling leads to a slow
        // down of the scrolling during the animation. Instead of changing the size of the scroll
        // view, the top margin of the content is changed which solves the animation issues.
        ZStack(alignment: .topLeading) {
            VStack(spacing: 16) {
                MultihopSelectionView(
                    hops: (viewModel.isMultihopActive ? MultihopContext.allCases : [MultihopContext.exit])
                        .map {
                            var selectedLocation: LocationNode?
                            var filterCount = 0
                            switch $0 {
                            case .entry:
                                selectedLocation =
                                    viewModel.showMultihopInfo
                                    ? AutomaticLocationNode(
                                        locationInfo: (viewModel.connectedEntryLocation.flatMap {
                                            [$0.country]
                                        }) ?? []
                                    )
                                    : viewModel.entryContext.selectedLocation
                                filterCount = viewModel.entryContext.filter.count
                            case .exit:
                                selectedLocation = viewModel.exitContext.selectedLocation
                                filterCount = viewModel.exitContext.filter.count
                            }
                            return Hop(
                                multihopContext: $0,
                                multihopState: viewModel.multihopState,
                                selectedLocation: selectedLocation,
                                filterCount: filterCount
                            )
                        },
                    selectedMultihopContext: $viewModel.multihopContext,
                    isExpanded: headerIsExpanded,
                    onFilterTapped: {
                        viewModel.showFilterView(context: $0)
                    }
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
            .padding(.horizontal, 24)
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
        .animation(.default, value: viewModel.isMultihopActive)
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
                        Picker(
                            selection: Binding(
                                get: { viewModel.multihopState },
                                set: { newValue in
                                    if viewModel.multihopStateIsIncompatible(newValue) {
                                        multihopBlockedStateWarningAlert = getMultihopBlockedStateWarningAlert(
                                            newMultihopState: newValue
                                        )
                                    } else {
                                        viewModel.multihopState = newValue
                                    }
                                }
                            )
                        ) {
                            ForEach(MultihopState.allCases, id: \.self) { state in
                                HStack {
                                    Text(state.description)
                                    state.icon
                                        .renderingMode(.template)
                                }
                                .accessibilityIdentifier(.multihopState(state.description))
                            }
                        } label: {
                            Text("Multihop mode")
                            Text(viewModel.multihopState.description)
                        }
                        .pickerStyle(MenuPickerStyle())
                        .accessibilityIdentifier(.multihopMenuPicker)

                        Button {
                            if viewModel.isRecentsEnabled {
                                disablingRecentConnectionsAlert = getDisableRecentsWarningAlert()
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
                                Image.mullvadIconReload
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
        .mullvadAlert(item: $multihopBlockedStateWarningAlert)
    }

    private func getMultihopBlockedStateWarningAlert(newMultihopState: MultihopState) -> MullvadAlert? {
        MullvadAlert(
            type: .warning,
            messages: [
                LocalizedStringKey(
                    String(
                        format: NSLocalizedString(
                            "Enabling “%@” will block your Internet connection due to "
                                + "incompatible settings. Do you wish to continue?", comment: ""
                        ),
                        NSLocalizedString(newMultihopState.description, comment: "One of three multihop states")
                    )
                )
            ],
            actions: [
                MullvadAlert.Action(
                    type: .danger,
                    title: "Enable",
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = newMultihopState
                        multihopBlockedStateWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .default,
                    title: "Cancel",
                    handler: {
                        multihopBlockedStateWarningAlert = nil
                    }
                ),
            ]
        )
    }

    private func getDisableRecentsWarningAlert() -> MullvadAlert {
        MullvadAlert(
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
