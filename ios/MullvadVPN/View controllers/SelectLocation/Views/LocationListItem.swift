import SwiftUI

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @Binding var location: LocationNode
    let selectedLocation: LocationNode?
    let connectedRelayHostname: String?
    let position: ItemPosition
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu
    var level = 0
    var shouldBeExpanded: Bool {
        if let selectedLocation {
            var curr = selectedLocation
            while let parent = curr.parent {
                if parent.code == location.code {
                    return true
                }
                curr = parent
            }
        }
        return location.showsChildren
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
                Button {
                    onSelect(location)
                } label: {
                    RelayItemView(
                        label: location.name,
                        isSelected: selectedLocation?.code == location.code,
                        isConnected: connectedRelayHostname == location.name,
                        position: position,
                        level: level
                    )
                }
                .disabled(!location.isActive)
            } else {
                LocationDisclosureGroup(
                    level: level,
                    position: position,
                    isActive: location.isActive,
                    isExpanded: $location.showsChildren
                ) {
                    ForEach(
                        Array(filteredChildrenIndices.enumerated()),
                        id: \.element
                    ) { index, indexInChildrenList in
                        let location = $location.children[indexInChildrenList]
                        LocationListItem(
                            location: location,
                            selectedLocation: selectedLocation,
                            connectedRelayHostname: connectedRelayHostname,
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
                    let isSelected = selectedLocation?.code == location.code
                    HStack {
                        if !location.isActive {
                            Image.mullvadRedDot
                        } else if isSelected {
                            Image.mullvadIconTick
                                .foregroundStyle(Color.mullvadSuccessColor)
                        }
                        Text(location.name)
                            .foregroundStyle(
                                location.isActive
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
                    location: .constant(.init(name: "test", code: "test")),
                    selectedLocation: nil,
                    connectedRelayHostname: nil,
                    position: .only,
                    onSelect: { _ in print("Got ya!") },
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
