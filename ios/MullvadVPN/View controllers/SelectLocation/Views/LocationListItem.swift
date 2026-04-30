import MullvadTypes
import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @State private var alert: MullvadAlert?
    @Environment(\.dismissSearchFocus) private var dismissSearchFocus
    private let itemFactory = ListItemFactory()

    @Binding var location: LocationNode
    var isLastInList: Bool = true
    let multihopContext: MultihopContext
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu
    var level = 0

    var childIndices: [Int] {
        location.children
            .enumerated()
            .map { $0.offset }
    }

    var body: some View {
        if location is AutomaticLocationNode {
            AutomaticLocationListItem(location: $location, isRecent: false, onSelect: onSelect)
        } else {
            locationListItem
        }
    }

    @ViewBuilder
    var locationListItem: some View {
        let hasChildren = !childIndices.isEmpty
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
                if hasChildren {
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
                        Array(childIndices.enumerated()),
                        id: \.element
                    ) { index, indexInChildrenList in
                        let location = $location.children[indexInChildrenList]
                        LocationListItem(
                            location: location,
                            isLastInList: isLastInList && index == (childIndices.count - 1),
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
            contextMenu(location)
        }
        .zIndex(level == 0 ? 2 : 1 / Double(level))  // prevent wrong overlapping during animations
        .id(location.id)  // to be able to scroll to this item programmatically
    }

    func toggleChildren() {
        dismissSearchFocus?()
        withAnimation(.default.speed(3)) {
            location.showsChildren.toggle()
        }
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
