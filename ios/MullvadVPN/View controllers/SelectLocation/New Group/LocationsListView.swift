import SwiftUI

struct LocationsListView: View {
    let locations: [LocationNode]

    var body: some View {
            VStack(spacing: 4) {
                ForEach(Array(locations.enumerated()), id: \.offset) { index, country in
                    LocationListItem(
                        location: country,
                        position: ItemPosition(
                            index: index,
                            count: locations.count
                        )
                    )
                }
        }
    }
}

struct LocationListItem: View {
    let location: LocationNode
    let position: ItemPosition
    var level: Int = 0
    var body: some View {
        if location.children.isEmpty {
            Button {
                print("Selected relay: \(location.name)")
            } label: {
                HStack {
                    RelayItemView(
                        label: location.name,
                        text: "Connected Server",
                        position: position,
                        level: level
                    )
                }
            }
        } else {
            LocationDisclosureGroup(
                level: level,
                position: position
            ) {
                ForEach(
                    Array(location.children.enumerated()),
                    id: \.offset
                ) { index, child in
                    LocationListItem(
                        location: child,
                        position: level > 0 && position != .last ? .middle : ItemPosition(
                            index: index + 1,
                            count: location.children.count + 1
                        ),
                        level: level + 1,
                    )
                }
            } label: {
                Text(location.name)
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .font(.mullvadSmallSemiBold)
                    .padding(.horizontal, CGFloat(16 * (level + 1)))
                    .padding(.vertical, 16)
            } onSelect: {
                print("Selected location: \(location.name)")
            }
        }
    }
}

private struct RelayItemView: View {
    let label: String
    let text: String?
    let position: ItemPosition
    let level: Int

    init(label: String, text: String? = nil, position: ItemPosition, level: Int) {
        self.label = label
        self.text = text
        self.position = position
        self.level = level
    }

    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                Text(label)
                    .font(.mullvadSmallSemiBold)
                    .foregroundStyle(Color.mullvadTextPrimary)
                if let text {
                    Text(text)
                        .font(.mullvadMiniSemiBold)
                        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                }
            }
            Spacer()
        }
        .padding(.vertical, text == nil ? 16 : 8)
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
    @State private var isExpanded = false

    let position: ItemPosition
    let level: Int
    let label: () -> Label
    let content: () -> Content
    let onSelect: (() -> Void)?

    init(
        level: Int,
        position: ItemPosition = .only,
        isExpanded: Bool? = nil,
        @ViewBuilder content: @escaping () -> Content,
        @ViewBuilder label: @escaping () -> Label,
        onSelect: (() -> Void)? = nil
    ) {
        self.position = position
        self.level = level
        self.isExpanded = isExpanded ?? false
        self.label = label
        self.content = content
        self.onSelect = onSelect
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
            .zIndex(1)

            if isExpanded {
                VStack(spacing: 1) {
                    content()
                }
                .padding(.top, 1)
            }
        }
    }
}

#Preview {
    var locations: [LocationNode] = [
        LocationNode(name: "Sweden", code: "se", children: [
            LocationNode(name: "Stockholm", code: "sth", children: [
                LocationNode(name: "se-sto-001", code: "se-sto-001"),
                LocationNode(name: "se-sto-002", code: "se-sto-002"),
                LocationNode(name: "se-sto-003", code: "se-sto-003")
            ]
                        ),
            LocationNode(name: "Gothenburg", code: "gto", children: [
                LocationNode(name: "se-got-001", code: "se-got-001"),
                LocationNode(name: "se-got-002", code: "se-got-002"),
                LocationNode(name: "se-got-003", code: "se-got-003")
            ]),
        ]),
        LocationNode(name: "blo-la-003", code: "blo-la-003"),
        LocationNode(name: "Germany", code: "de", children: [
            LocationNode(name: "Berlin", code: "ber", children: [
                LocationNode(name: "de-ber-001", code: "de-ber-001"),
                LocationNode(name: "de-ber-002", code: "de-ber-002"),
                LocationNode(name: "de-ber-003", code: "de-ber-003")
            ]),
            LocationNode(name: "Frankfurt", code: "fra", children: [
                LocationNode(name: "de-fra-001", code: "de-fra-001"),
                LocationNode(name: "de-fra-002", code: "de-fra-002"),
                LocationNode(name: "de-fra-003", code: "de-fra-003")
            ]),
        ]),
        LocationNode(name: "France", code: "fr", children: [
            LocationNode(name: "Paris", code: "par", children: [
                LocationNode(name: "fr-par-001", code: "fr-par-001"),
                LocationNode(name: "fr-par-002", code: "fr-par-002"),
                LocationNode(name: "fr-par-003", code: "fr-par-003")
            ]),
            LocationNode(name: "Lyon", code: "lyo", children: [
                LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                LocationNode(name: "fr-lyo-003", code: "fr-lyo-003")
            ]),
        ]),
        LocationNode(name: "Lalala", code: "test"),
        LocationNode(name: "Custom list", code: "blda", children: [
            LocationNode(name: "de-ber-003", code: "de-ber-003"),

            LocationNode(name: "France", code: "fr", children: [
                LocationNode(name: "Paris", code: "par", children: [
                    LocationNode(name: "fr-par-001", code: "fr-par-001"),
                    LocationNode(name: "fr-par-002", code: "fr-par-002"),
                    LocationNode(name: "fr-par-003", code: "fr-par-003")
                ]),
                LocationNode(name: "Lyon", code: "lyo", children: [
                    LocationNode(name: "fr-lyo-001", code: "fr-lyo-001"),
                    LocationNode(name: "fr-lyo-002", code: "fr-lyo-002"),
                    LocationNode(name: "fr-lyo-003", code: "fr-lyo-003")
                ]),
            ]),
            LocationNode(name: "testserver", code: "1234")
        ]),
]
    ScrollView {
        LocationsListView(
            locations: locations,
        )
        .padding()
    }
    .background(Color.mullvadBackground)
}
