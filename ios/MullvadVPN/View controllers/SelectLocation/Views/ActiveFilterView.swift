import SwiftUI

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let onSelect: (SelectLocationFilter) -> Void
    let onRemove: (SelectLocationFilter) -> Void
    @State private var maxItemHeight: CGFloat = 0

    // Show filters that can't be removed to the left
    private var sortedFilters: [SelectLocationFilter] {
        activeFilter
            .sorted {
                !$0.canBeRemoved && $1.canBeRemoved
            }
    }
    var body: some View {
        ScrollView(.horizontal) {
            HStack {
                ForEach(sortedFilters, id: \.hashValue) { filter in
                    Button {
                        onSelect(filter)
                    } label: {
                        HStack {
                            Text(filter.title)
                                .font(.mullvadMiniSemiBold)
                                .foregroundStyle(Color.mullvadTextPrimary)
                            if filter.canBeRemoved {
                                Button {
                                    onRemove(filter)
                                } label: {
                                    Image.mullvadIconCross
                                }
                                .accessibilityIdentifier(.relayFilterChipCloseButton)
                            }
                        }
                        .padding(8)
                        .sizeOfView { size in
                            maxItemHeight = max(maxItemHeight, size.height)
                        }
                        .frame(height: maxItemHeight)
                        .background {
                            RoundedRectangle(cornerRadius: 8)
                                .foregroundStyle(Color.MullvadButton.primary)
                        }
                    }
                    .accessibilityIdentifier(filter.accessibilityIdentifier)
                }
            }
        }
        .apply {
            if #available(iOS 16.0, *) {
                $0.scrollIndicators(.never)
            } else {
                $0
            }
        }
    }
}

#Preview {
    Text("da")
        .sheet(isPresented: .constant(true)) {
            NavigationView {
                ScrollView {
                    ActiveFilterView(
                        activeFilter: [
                            .daita,
                            .owned,
                            .rented,
                            .provider(2),
                            .obfuscation,
                        ],
                        onSelect: { _ in
                        },
                        onRemove: { _ in }
                    )
                }
            }
        }
}
