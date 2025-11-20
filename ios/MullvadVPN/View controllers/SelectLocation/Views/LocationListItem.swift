import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
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
        Group {
            if location.children.isEmpty {
                RelayItemView(
                    location: location,
                    multihopContext: multihopContext,
                    level: level,
                    isLastInList: isLastInList,
                    onSelect: { onSelect(location) }
                )
                .accessibilityIdentifier(.locationListItem(location.name))
                .contextMenu {
                    contextMenu(location)
                }
                .padding(.top, level == 0 ? 0 : 1)
            } else {
                LocationDisclosureGroup(
                    level: level,
                    isLastInList: isLastInList,
                    isActive: location.isActive && !location.isExcluded,
                    isExpanded: $location.showsChildren,
                    contextMenu: { contextMenu(location) },
                    accessibilityIdentifier: .locationListItem(location.name)
                ) {
                    ForEach(
                        Array(filteredChildrenIndices.enumerated()),
                        id: \.element
                    ) { index, indexInChildrenList in
                        let location = $location.children[indexInChildrenList]
                        LocationListItem(
                            location: location,
                            isLastInList: isLastInList && index == (filteredChildrenIndices.count - 1),
                            multihopContext: multihopContext,
                            onSelect: onSelect,
                            contextMenu: { location in contextMenu(location) },
                            level: level + 1,
                        )
                    }
                } label: {
                    HStack {
                        if !location.isActive {
                            Image.mullvadRedDot
                        } else if location.isSelected {
                            Image.mullvadIconTick
                                .foregroundStyle(Color.mullvadSuccessColor)
                        }
                        Text(location.name)
                            .foregroundStyle(
                                location.isActive && !location.isExcluded
                                    ? location.isSelected
                                        ? Color.mullvadSuccessColor
                                        : Color.mullvadTextPrimary
                                    : Color.mullvadTextPrimaryDisabled
                            )
                            .font(.mullvadSmallSemiBold)
                            .multilineTextAlignment(.leading)
                    }
                    .padding(.leading, CGFloat(16 * (level + 1)))
                    .padding(.trailing, 8)
                    .padding(.vertical, 16)
                } onSelect: {
                    onSelect(location)
                }
            }
        }
        .zIndex(level == 0 ? 2 : 1 / Double(level))  // prevent wrong overlapping during animations
        .id(location.code)  // to be able to scroll to this item programmatically
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
