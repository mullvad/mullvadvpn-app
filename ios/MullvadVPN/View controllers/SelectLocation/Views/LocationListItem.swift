import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @Binding var location: LocationNode
    let multihopContext: MultihopContext
    let position: ItemPosition
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
                    position: position,
                    level: level,
                    onSelect: { onSelect(location) }
                )
                .accessibilityIdentifier(.locationListItem(location.name))
            } else {
                LocationDisclosureGroup(
                    level: level,
                    position: position,
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
                            position: level > 0 && position != .last
                                ? .middle
                                : ItemPosition(
                                    index: index + 1,
                                    count: filteredChildrenIndices.count + 1
                                ),
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
        .contextMenu {
            contextMenu(location)
        }
    }
}

enum ItemPosition: String {
    case first
    case middle
    case last
    case only

    init(index: Int, count: Int) {
        if index == 0 {
            if count == 1 {
                self = .only
            } else {
                self = .first
            }
        } else if index == count - 1 {
            self = .last
        } else {
            self = .middle
        }
    }
}

@available(iOS 17, *)
#Preview {
    @Previewable @State var disabled: Bool = false
    Text("")
        .sheet(isPresented: .constant(true)) {
            ScrollView {
                LocationListItem(
                    location:
                        .constant(
                            .init(name: "test", code: "test")
                        ),
                    multihopContext: .exit,
                    position: .only,
                    onSelect: { _ in },
                    contextMenu: { _ in EmptyView() },
                    level: 0
                )
                .disabled(disabled)
            }
            .gesture(
                DragGesture().onChanged({ _ in disabled.toggle() }),
                including: .none
            )
        }
}
