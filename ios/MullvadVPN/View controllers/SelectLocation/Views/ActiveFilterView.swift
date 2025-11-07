import SwiftUI

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let onSelect: (SelectLocationFilter) -> Void
    let onRemove: (SelectLocationFilter) -> Void

    // Show filters that can't be removed to the left
    private var sortedFilters: [SelectLocationFilter] {
        activeFilter
            .sorted {
                !$0.isRemovable && $1.isRemovable
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
                            if filter.isRemovable {
                                Button {
                                    onRemove(filter)
                                } label: {
                                    Image(systemName: "xmark")
                                }
                                .accessibilityIdentifier(.relayFilterChipCloseButton)
                            }
                        }
                        .foregroundStyle(Color.mullvadTextPrimary)
                        .font(.mullvadMiniSemiBold)
                        .padding(8)
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
    Text("")
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
