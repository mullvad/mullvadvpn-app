import SwiftUI

struct LocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [LocationNode]
    let selectedLocation: LocationNode?
    let connectedRelayHostname: String?
    let onSelectLocation: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var filteredLocationIndices: [Int] {
        locations
            .enumerated()
            .filter { !$0.element.isHiddenFromSearch }
            .map { $0.offset }
    }

    var body: some View {
        VStack(spacing: 4) {
            ForEach(
                Array(filteredLocationIndices.enumerated()),
                id: \.element
            ) { index, indexInLocationList in
                let location = $locations[indexInLocationList]
                LocationListItem(
                    location: location,
                    selectedLocation: selectedLocation,
                    connectedRelayHostname: connectedRelayHostname,
                    position: ItemPosition(
                        index: index,
                        count: filteredLocationIndices.count
                    ),
                    onSelect: onSelectLocation,
                    contextMenu: { contextMenu(location.wrappedValue) }
                )
            }
        }
    }
}

struct LocationListItem<ContextMenu>: View where ContextMenu: View {
    @Binding var location: LocationNode
    let selectedLocation: LocationNode?
    let connectedRelayHostname: String?
    let position: ItemPosition
    let onSelect: (LocationNode) -> Void
    let contextMenu: () -> ContextMenu
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
                    HStack {
                        RelayItemView(
                            label: location.name,
                            isSelected: selectedLocation?.code == location.code,
                            isConnected: connectedRelayHostname == location.name,
                            position: position,
                            level: level
                        )
                    }
                }
            } else {
                LocationDisclosureGroup(
                    level: level,
                    position: position,
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
                            contextMenu: { contextMenu() },
                            level: level + 1,
                        )
                    }
                } label: {
                    let isSelected = selectedLocation?.code == location.code
                    HStack {
                        if isSelected {
                            Image.mullvadIconTick
                                .foregroundStyle(Color.mullvadSuccessColor)
                        }
                        Text(location.name)
                            .foregroundStyle(
                                isSelected ? Color.mullvadSuccessColor : Color.mullvadTextPrimary
                            )
                            .font(.mullvadSmallSemiBold)
                    }
                    .padding(.horizontal, CGFloat(16 * (level + 1)))
                    .padding(.vertical, 16)
                } onSelect: {
                    onSelect(location)
                }
            }
        }
        .transformEffect(.identity)
        .contextMenu {
            contextMenu()
        }
    }
}

private struct RelayItemView: View {
    let label: String
    let isSelected: Bool
    let isConnected: Bool
    let position: ItemPosition
    let level: Int

    var showSubtitle: Bool {
        !isSelected && isConnected
    }

    init(label: String, isSelected: Bool, isConnected: Bool, position: ItemPosition, level: Int) {
        self.label = label
        self.isSelected = isSelected
        self.isConnected = isConnected
        self.position = position
        self.level = level
    }

    var body: some View {
        HStack {
            if isSelected {
                Image.mullvadIconTick
                    .foregroundStyle(Color.mullvadSuccessColor)
            }
            VStack(alignment: .leading) {
                Text(label)
                    .font(.mullvadSmallSemiBold)
                    .foregroundStyle(isSelected
                        ? Color.mullvadSuccessColor
                        : Color.mullvadTextPrimary
                    )
                if showSubtitle {
                    Text("Connected server")
                        .font(.mullvadMiniSemiBold)
                        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                }
            }
            Spacer()
        }
        .padding(.vertical, showSubtitle ? 8 : 16)
        .padding(.horizontal, CGFloat(16 * (level + 1)))
        .background {
            let backgroundColor = Color.colorForLevel(level)
            let corners: UIRectCorner =
                if level == 0 {
                    .allCorners
                } else {
                    switch position {
                    case .only: .allCorners
                    case .first: []
                    case .middle: []
                    case .last: [.bottomLeft, .bottomRight]
                    }
                }
            MullvadRoundedCorner(
                cornerRadius: 16,
                corners: corners
            )
            .foregroundStyle(backgroundColor)
        }
    }
}

fileprivate extension Color {
    static func colorForLevel(_ level: Int) -> Color {
        switch level {
        case 1: Color.MullvadList.Item.child1
        case 2: Color.MullvadList.Item.child2
        case 3: Color.MullvadList.Item.child3
        case 4: Color.MullvadList.Item.child4
        default: Color.MullvadList.Item.parent
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

private struct LocationDisclosureGroup<Label: View, Content: View>: View {
    @Binding private var isExpanded: Bool

    let position: ItemPosition
    let level: Int
    let label: () -> Label
    let content: () -> Content
    let onSelect: (() -> Void)?
    let onLongPress: (() -> Void)?

    init(
        level: Int,
        position: ItemPosition = .only,
        isExpanded: Binding<Bool>,
        @ViewBuilder content: @escaping () -> Content,
        @ViewBuilder label: @escaping () -> Label,
        onSelect: (() -> Void)? = nil,
        onLongPress: (() -> Void)? = nil
    ) {
        self.position = position
        self.level = level
        self._isExpanded = isExpanded

        self.label = label
        self.content = content
        self.onSelect = onSelect
        self.onLongPress = onLongPress
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 2) {
                Button {
                    onSelect?()
                } label: {
                    HStack {
                        label()
                        Spacer()
                    }
                    .frame(maxHeight: .infinity)
                    .background {
                        let backgroundColor = Color.colorForLevel(level)
                        let corners: UIRectCorner =
                            if level == 0 {
                                if isExpanded {
                                    [.topLeft]
                                } else {
                                    [.topLeft, .bottomLeft]
                                }
                            } else {
                                switch position {
                                case .only: [.topLeft, .bottomLeft]
                                case .first: [.topLeft]
                                case .middle: []
                                case .last: isExpanded ? [] : [.bottomLeft]
                                }
                            }
                        MullvadRoundedCorner(cornerRadius: 16, corners: corners)
                            .foregroundStyle(backgroundColor)
                    }
                }
                Button {
                    withAnimation {
                        isExpanded.toggle()
                    }
                } label: {
                    Image.mullvadIconChevron
                        .rotationEffect(.degrees(isExpanded ? -90 : 90))
                        .animation(.default, value: isExpanded)
                        .padding(16)
                        .background {
                            let corners: UIRectCorner =
                                if level == 0 {
                                    if isExpanded {
                                        [.topRight]
                                    } else {
                                        [.topRight, .bottomRight]
                                    }
                                } else {
                                    switch position {
                                    case .only: [.topRight, .bottomRight]
                                    case .first: [.topRight]
                                    case .middle: []
                                    case .last: isExpanded ? [] : [.bottomRight]
                                    }
                                }
                            MullvadRoundedCorner(
                                cornerRadius: 16,
                                corners: corners
                            )
                            .foregroundStyle(Color.MullvadList.Item.parent)
                        }
                        .frame(maxHeight: .infinity)
                }
                .contentShape(Rectangle())
            }

            if isExpanded {
                VStack(spacing: 1) {
                    content()
                }
                .padding(.top, 1)
            }
        }
    }
}

@available(iOS 17, *)
#Preview {
    @Previewable @State var locations: [LocationNode] = [
        LocationNode(name: "Sweden", code: "se", children: [
            LocationNode(
                name: "Stockholm",
                code: "sth",
                children: [
                    LocationNode(name: "se-sto-001", code: "se-sto-001"),
                    LocationNode(name: "se-sto-002", code: "se-sto-002"),
                    LocationNode(name: "se-sto-003", code: "se-sto-003"),
                ]
            ),
            LocationNode(name: "Gothenburg", code: "gto", children: [
                LocationNode(name: "se-got-001", code: "se-got-001"),
                LocationNode(name: "se-got-002", code: "se-got-002"),
                LocationNode(name: "se-got-003", code: "se-got-003"),
            ]),
        ]),
        LocationNode(name: "blo-la-003", code: "blo-la-003"),
        LocationNode(name: "Germany", code: "de", children: [
            LocationNode(name: "Berlin", code: "ber", children: [
                LocationNode(name: "de-ber-001", code: "de-ber-001"),
                LocationNode(name: "de-ber-002", code: "de-ber-002"),
                LocationNode(name: "de-ber-003", code: "de-ber-003"),
            ]),
            LocationNode(name: "Frankfurt", code: "fra", children: [
                LocationNode(name: "de-fra-001", code: "de-fra-001"),
                LocationNode(name: "de-fra-002", code: "de-fra-002"),
                LocationNode(name: "de-fra-003", code: "de-fra-003"),
            ]),
        ]),
        LocationNode(name: "France", code: "fr", children: [
            LocationNode(name: "Paris", code: "par", children: [
                LocationNode(name: "fr-par-001", code: "fr-par-001"),
                LocationNode(name: "fr-par-002", code: "fr-par-002"),
                LocationNode(name: "fr-par-003", code: "fr-par-003"),
            ]),
            LocationNode(name: "Lyon", code: "lyo", children: [
                LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
            ]),
        ]),
        LocationNode(name: "Lalala", code: "test"),
        LocationNode(name: "Custom list", code: "blda", children: [
            LocationNode(name: "de-ber-003", code: "de-ber-003"),

            LocationNode(name: "France", code: "fr", children: [
                LocationNode(name: "Paris", code: "par", children: [
                    LocationNode(name: "fr-par-001", code: "fr-par-001"),
                    LocationNode(name: "fr-par-002", code: "fr-par-002"),
                    LocationNode(name: "fr-par-003", code: "fr-par-003"),
                ]),
                LocationNode(name: "Lyon", code: "lyo", children: [
                    LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                    LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                    LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
                ]),
            ]),
            LocationNode(name: "testserver", code: "1234"),
        ]),
    ]
    ScrollView {
        LocationsListView(
            locations: $locations,
            selectedLocation: LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
            connectedRelayHostname: "fr-lyo-003",
            onSelectLocation: { location in
                print("Selected: \(location.name)")
            },
            contextMenu: { _ in Text("Add to list") }
        )
        .padding()
    }
    .background(Color.mullvadBackground)
}

@available(iOS 17, *)
#Preview {
    @Previewable @State var location = LocationNode(name: "Custom list", code: "blda", children: [
        LocationNode(name: "de-ber-003", code: "de-ber-003"),

        LocationNode(name: "France", code: "fr", children: [
            LocationNode(name: "Paris", code: "par", children: [
                LocationNode(name: "fr-par-001", code: "fr-par-001"),
                LocationNode(name: "fr-par-002", code: "fr-par-002"),
                LocationNode(name: "fr-par-003", code: "fr-par-003"),
            ], showsChildren: true),
            LocationNode(name: "Lyon", code: "lyo", children: [
                LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                LocationNode(name: "fr-lyo-003", code: "fr-lyo-003"),
            ]),
        ], showsChildren: true),
        LocationNode(name: "testserver", code: "1234"),
    ], showsChildren: true)
    ScrollView {
        LocationListItem(
            location: $location,
            selectedLocation: nil,
            connectedRelayHostname: nil,
            position: .only,
            onSelect: { _ in },
            contextMenu: { Text("Add to list") },
            level: 0
        )
    }
}
