import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @Binding var location: LocationNode
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
                    onSelect: { onSelect(location) }
                )
                .accessibilityIdentifier(.locationListItem(location.name))
            } else {
                LocationDisclosureGroup(
                    level: level,
                    isActive: location.isActive && !location.isExcluded,
                    isExpanded: $location.showsChildren,
                    accessibilityIdentifier: .locationListItem(location.name)
                ) {
                    ForEach(
                        Array(filteredChildrenIndices.enumerated()),
                        id: \.element
                    ) { index, indexInChildrenList in
                        let location = $location.children[indexInChildrenList]
                        LocationListItem(
                            location: location,
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
        .id(location.code)  // to be able to scroll to this item programmatically
        .transformEffect(.identity)
        .apply {
            if level == 0 {
                $0.clipShape(RoundedRectangle(cornerRadius: 16))
            } else {
                $0
            }
        }
        .contextMenu {
            contextMenu(location)
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
                    location: .constant(viewModel.exitContext.locations.first!),
                    multihopContext: .exit,
                    onSelect: { _ in },
                    contextMenu: { _ in EmptyView() },
                    level: 0
                )
            }
        }
}
