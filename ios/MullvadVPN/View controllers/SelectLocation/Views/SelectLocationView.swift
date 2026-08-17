import MullvadSettings
import MullvadTypes
import SwiftUI

struct SelectLocationView<ViewModel>: View where ViewModel: SelectLocationViewModel {
    @ObservedObject var viewModel: ViewModel
    @State private var isScrolledToTop: Bool = true
    @State private var disablingRecentConnectionsAlert: MullvadAlert?
    @State private var multihopWarningAlert: MullvadAlert?
    @FocusState private var focusSearchField: Bool
    @State private var isSearchExpanded: Bool = false
    @State private var headerHeight: CGFloat = 0
    @State private var floatingBarHeight: CGFloat = 0
    @ScaledMetric(relativeTo: .body) private var listBottomInset: CGFloat = 56

    private var headerIsExpanded: Bool {
        switch viewModel.multihopContext {
        case .entry:
            viewModel.showMultihopInfo || isScrolledToTop
        case .exit:
            isScrolledToTop
        }
    }

    private var showSearchField: Bool {
        return !viewModel.showMultihopInfo || viewModel.multihopContext == .exit
    }

    var body: some View {
        VStack(spacing: 0) {
            // Eventhough the location list is not in the top,
            // the navigation bar would changes appearence when the list gets scrolled.
            // (see UINavigationBar+Appearance.swift)
            // Adding an empty scroll view on top prevents that.
            ScrollView {}.frame(height: 0)

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
            .padding(.vertical, 8)

            Group {
                switch viewModel.multihopContext {
                case .exit:
                    VStack {
                        if !viewModel.exitContext.filter.isEmpty && headerIsExpanded {
                            activeFilterView(
                                locationContext: viewModel.exitContext
                            )
                        }
                        ExitLocationView(
                            viewModel: viewModel,
                            context: $viewModel.exitContext,
                            onScrollVisibilityChange: {
                                expandHeader in
                                if viewModel.multihopContext == .exit {
                                    withAnimation {
                                        isScrolledToTop = expandHeader
                                    }
                                }
                            }
                        )
                        .contentMargins(.bottom, showSearchField ? floatingBarHeight + listBottomInset : 0)
                    }
                    .transition(.move(edge: .trailing).combined(with: .opacity))
                case .entry:
                    VStack {
                        if !viewModel.entryContext.filter.isEmpty && headerIsExpanded {
                            activeFilterView(
                                locationContext: viewModel.entryContext
                            )
                        }
                        EntryLocationView(
                            viewModel: viewModel,
                            onScrollVisibilityChange: {
                                expandHeader in
                                if viewModel.multihopContext == .entry {
                                    withAnimation {

                                        isScrolledToTop = expandHeader
                                    }

                                }
                            }
                        )
                        .contentMargins(.bottom, showSearchField ? floatingBarHeight + listBottomInset : 0)
                    }
                    .transition(.move(edge: .leading).combined(with: .opacity))
                }
            }
            .simultaneousGesture(
                DragGesture(minimumDistance: 50)
                    .onChanged { _ in
                        focusSearchField = false
                    }
            )
            .environment(\.dismissSearchFocus, { focusSearchField = false })
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
                                    if viewModel.filtersWillBeOverridden(newValue) {
                                        multihopWarningAlert = getMultihopFilterOverrideWarningAlert(
                                            newMultihopState: newValue
                                        )
                                    } else if viewModel.multihopStateIsIncompatible(newValue) {
                                        multihopWarningAlert = getMultihopBlockedStateWarningAlert(
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
        .mullvadAlert(item: $multihopWarningAlert)
    }

    private func activeFilterView(locationContext: LocationContext) -> some View {
        ActiveFilterView(
            activeFilter: viewModel.visibleFilterChips,
            labelStyle: .general,
            automaticLocationIsActive: locationContext.isAutomaticLocation,
            shouldShowAutomaticFilterOverrideNotice: locationContext.isFilterOverridden
        ) { filter in
            viewModel.onFilterTapped(filter)
        } onRemove: { filter in
            viewModel.onFilterRemoved(filter)
        }
        .transition(.move(edge: .top).combined(with: .opacity))
    }

    private func getMultihopFilterOverrideWarningAlert(newMultihopState: MultihopState) -> MullvadAlert? {
        MullvadAlert(
            type: .warning,
            messages: [
                LocalizedStringKey(
                    String(
                        format: NSLocalizedString(
                            "You currently have entry filters applied. Switching to “%@“, the app will ignore filter "
                                + "settings for the entry server that is being automatically selected.",
                            comment: "Variable refers to multihop mode"
                        ),
                        newMultihopState.description
                    )
                )
            ],
            actions: [
                MullvadAlert.Action(
                    type: .primary,
                    title: "Continue",
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = newMultihopState
                        multihopWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        multihopWarningAlert = nil
                    }
                ),
            ]
        )
    }

    private func getMultihopBlockedStateWarningAlert(newMultihopState: MultihopState) -> MullvadAlert? {
        MullvadAlert(
            type: .warning,
            messages: [LocalizedStringKey(BlockedStateString.Message.multihop.description)],
            actions: [
                MullvadAlert.Action(
                    type: .destructivePrimary,
                    title: LocalizedStringKey(BlockedStateString.Button.multihop(newMultihopState).description),
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = newMultihopState
                        multihopWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        multihopWarningAlert = nil
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
                    type: .destructivePrimary,
                    title: "Disable",
                    identifier: AccessibilityIdentifier.disableRecentConnectionsButton,
                    handler: {
                        disablingRecentConnectionsAlert = nil
                        viewModel.toggleRecents()
                    }
                ),
                MullvadAlert.Action(
                    type: .secondary,
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
