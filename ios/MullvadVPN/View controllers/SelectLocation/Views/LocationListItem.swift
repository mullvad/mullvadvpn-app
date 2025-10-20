import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @Binding var location: LocationNode
    let multihopContext: MultihopContext
    let position: ItemPosition
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu
    var level = 0
    var shouldBeExpanded: Bool {
        var childIsSelected = false
        location.forEachDescendant { child in
            let isSelected =
                switch multihopContext {
                case .entry:
                    child.isSelected == .entry
                case .exit:
                    child.isSelected == .exit
                }
            if isSelected {
                childIsSelected = true
            }
        }
        return location.showsChildren || childIsSelected
    }

    var isExcluded: Bool {
        switch multihopContext {
        case .entry:
            return location.isExcludedFrom == .entry
        case .exit:
            return location.isExcludedFrom == .exit
        }
    }

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
            } else {
                LocationDisclosureGroup(
                    level: level,
                    position: position,
                    isActive: location.isActive && !isExcluded,
                    isExpanded: $location.showsChildren
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
                    let isSelected =
                        switch multihopContext {
                        case .entry:
                            location.isSelected == .entry
                        case .exit:
                            location.isSelected == .exit
                        }
                    HStack {
                        if !location.isActive {
                            Image.mullvadRedDot
                        } else if isSelected {
                            Image.mullvadIconTick
                                .foregroundStyle(Color.mullvadSuccessColor)
                        }
                        Text(location.name)
                            .foregroundStyle(
                                // TODO: FIX Color when excluded
                                location.isActive && location.isExcludedFrom == .none
                                    ? isSelected
                                        ? Color.mullvadSuccessColor
                                        : Color.mullvadTextPrimary
                                    : Color.mullvadTextPrimaryDisabled
                            )
                            .font(.mullvadSmallSemiBold)
                    }
                    .padding(.leading, CGFloat(16 * (level + 1)))
                    .padding(.trailing, 8)
                    .padding(.vertical, 16)
                } onSelect: {
                    onSelect(location)
                }
            }
        }
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
                    onSelect: { _ in print("Got ya!")
                    },
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
