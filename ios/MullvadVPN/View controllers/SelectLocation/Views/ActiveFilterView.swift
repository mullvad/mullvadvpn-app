import SwiftUI

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let disabled: Bool
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
                                .foregroundStyle(
                                    disabled
                                        ? Color.MullvadText.disabled
                                        : Color.mullvadTextPrimary
                                )
                            if filter.isRemovable {
                                Button {
                                    onRemove(filter)
                                } label: {
                                    Image(systemName: "xmark")
                                        .tint(Color.mullvadTextPrimary)
                                }
                                .accessibilityIdentifier(.relayFilterChipCloseButton)
                            }
                        }
                        .font(.mullvadMiniSemiBold)
                        .padding(8)
                        .background {
                            RoundedRectangle(cornerRadius: 8)
                                .foregroundStyle(
                                    disabled
                                        ? Color.MullvadButton.primaryDisabled
                                        : Color.MullvadButton.primary
                                )
                        }
                    }
                    .accessibilityIdentifier(filter.accessibilityIdentifier)
                }
            }
            .padding(.horizontal)
        }
        .scrollIndicators(.never)

        if disabled {
            Text("Filters are disabled when entry location is set to automatic")
                .font(.mullvadMini)
                .foregroundStyle(Color.MullvadText.onBackground)
                .padding(.horizontal)
                .padding(.top, -4)
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
                        disabled: true,
                        onSelect: { _ in },
                        onRemove: { _ in }
                    )
                    .background(Color.mullvadBackground)
                }
            }
        }
}
