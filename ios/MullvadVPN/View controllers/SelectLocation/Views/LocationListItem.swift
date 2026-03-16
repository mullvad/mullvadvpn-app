import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @State private var alert: MullvadAlert?
    private let itemFactory = ListItemFactory()

    @Binding var location: LocationNode
    var isLastInList: Bool = true
    let multihopContext: MultihopContext
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu
    var level = 0

    var filteredChildrenIndices: [Int] {
        location.children
            .enumerated()
            .filter { !$0.element.isHiddenFromSearch }
            .map { $0.offset }
    }

    var body: some View {
        let isAutomaticLocation = location is AutomaticLocationNode
        let childrenIndices = filteredChildrenIndices
        let hasChildren = !childrenIndices.isEmpty
        let isExpanded = location.showsChildren
        let isDisabled = !location.isActive || location.isExcluded

        SegmentedListItem(
            level: level,
            isLastInList: isLastInList,
            isDisabled: isDisabled,
            accessibilityIdentifier: .locationListItem(location.name),
            accessibilityLabel: location.name,
            label: {
                itemFactory.label(for: .location(node: location, context: multihopContext, level: level))
            },
            segment: {
                if isAutomaticLocation {
                    itemFactory.segment(
                        for: .info(
                            onSelect: {
                                alert = getAutomaticLocationInfoAlert { alert = nil }
                            }
                        )
                    )
                } else if hasChildren {
                    itemFactory.segment(
                        for: .expand(
                            isExpanded: isExpanded,
                            onSelect: {
                                toggleChildren()
                            }
                        )
                    )
                }
            },
            groupedContent: {
                if isExpanded {
                    ForEach(
                        Array(childrenIndices.enumerated()),
                        id: \.element
                    ) { index, indexInChildrenList in
                        let location = $location.children[indexInChildrenList]
                        LocationListItem(
                            location: location,
                            isLastInList: isLastInList && index == (childrenIndices.count - 1),
                            multihopContext: multihopContext,
                            onSelect: onSelect,
                            contextMenu: { location in contextMenu(location) },
                            level: level + 1,
                        )
                    }
                }
            },
            onSelect: { onSelect(location) }
        )
        .if(hasChildren) { view in
            view
                .accessibilityValue(isExpanded ? Text("Expanded") : Text("Collapsed"))
                .accessibilityAction(
                    named: isExpanded ? Text("Collapse \(location.name)") : Text("Expand \(location.name)")
                ) {
                    toggleChildren()
                }
        }
        .contextMenu {
            isAutomaticLocation ? nil : contextMenu(location)
        }
        .zIndex(level == 0 ? 2 : 1 / Double(level))  // prevent wrong overlapping during animations
        .id(location.id)  // to be able to scroll to this item programmatically
        .mullvadAlert(item: $alert)
    }

    func toggleChildren() {
        withAnimation(.default.speed(3)) {
            location.showsChildren.toggle()
        }
    }

    func getAutomaticLocationInfoAlert(completion: @escaping () -> Void) -> MullvadAlert {
        let message = [
            (NSLocalizedString(
                "Picks a suitable location based on your exit location, this is based on a number "
                    + "of different factors such as distance, provider, and server load.", comment: "")),
            (NSLocalizedString(
                "Attention: This will ignore filter settings for the server that is being "
                    + "used as an entry point", comment: "")),
        ].joinedParagraphs()

        return MullvadAlert(
            type: .info,
            messages: [LocalizedStringKey(message)],
            actions: [
                MullvadAlert.Action(
                    type: .default,
                    title: "Got it!",
                    handler: completion
                )
            ]
        )
    }
}

#Preview {
    let viewModel = MockSelectLocationViewModel()
    Text("")
        .sheet(isPresented: .constant(true)) {
            ScrollView {
                LocationListItem(
                    location:
                        .constant(
                            .init(name: "test", code: "test")
                        ),
                    multihopContext: .exit,
                    onSelect: { _ in },
                    contextMenu: { _ in EmptyView() },
                    level: 0
                )
                LocationListItem(
                    location:
                        .constant(
                            viewModel.exitContext.locations.first!
                        ),
                    multihopContext: .exit,
                    onSelect: { _ in
                    },
                    contextMenu: { _ in EmptyView() },
                    level: 0
                )
            }
        }
}
