import SwiftUI

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let labelStyle: SelectLocationFilter.LabelStyle
    let automaticLocationIsActive: Bool
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
                                .foregroundStyle(Color.MullvadButton.primary)
                        }
                    }
                    .accessibilityIdentifier(filter.accessibilityIdentifier)
                }
            }
            .padding(.horizontal)
        }
        .scrollIndicators(.never)

        if automaticLocationIsActive && activeFilter.contains(where: { $0.isOverriddenByAutomaticLocation }) {
            HStack(alignment: .center, spacing: 8) {
                UIImage.Buttons.info
                    .scaledIcon(fromBaseSize: 14, to: .footnote)
                    .foregroundStyle(Color.white)
                Text("Filters are overridden when using an automatic location")
                    .font(.mullvadMini)
                    .foregroundStyle(Color.MullvadText.onBackground)
                Spacer()
            }
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
                            .obfuscation(.lwo),
                        ],
                        labelStyle: .general,
                        automaticLocationIsActive: true,
                        onSelect: { _ in },
                        onRemove: { _ in }
                    )
                    .background(Color.mullvadBackground)
                }
            }
        }
}
