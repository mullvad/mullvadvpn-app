import SwiftUI

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let labelStyle: SelectLocationFilter.LabelStyle
    let automaticLocationIsActive: Bool
    let shouldShowAutomaticFilterOverrideNotice: Bool
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
        VStack(alignment: .leading, spacing: sortedFilters.isEmpty ? 8 : 16) {
            ScrollView(.horizontal) {
                if !activeFilter.isEmpty {
                    HStack {
                        ForEach(sortedFilters, id: \.hashValue) { filter in
                            Button {
                                onSelect(filter)
                            } label: {
                                HStack {
                                    Text(filter.labelText(style: labelStyle))
                                        .foregroundStyle(
                                            automaticLocationIsActive && filter.isOverriddenByAutomaticLocation
                                                ? Color.MullvadText.disabled
                                                : Color.mullvadTextPrimary
                                        )
                                    if filter.isRemovable {
                                        Button {
                                            onRemove(filter)
                                        } label: {
                                            ResizableImageView(image: .mullvadIconCross, dimension: .width(18))
                                        }
                                        .accessibilityIdentifier(.relayFilterChipCloseButton)
                                    }
                                }
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
                    .padding(.horizontal, 16)
                }
            }
            .scrollIndicators(.never)

            if shouldShowAutomaticFilterOverrideNotice {
                HStack(spacing: 4) {
                    ResizableImageView(image: .mullvadIconInfo, dimension: .width(14), tint: Color.mullvadTextSecondary)
                    Text("Filters are overridden when using an automatic location")
                        .font(.mullvadMini)
                        .foregroundStyle(Color.MullvadText.onBackground)
                }
                .padding(.horizontal, 16)
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
                            .obfuscation(.lwo),
                        ],
                        labelStyle: .general,
                        automaticLocationIsActive: true,
                        shouldShowAutomaticFilterOverrideNotice: true,
                        onSelect: { _ in },
                        onRemove: { _ in }
                    )
                    .background(Color.mullvadBackground)
                }
            }
        }
}
