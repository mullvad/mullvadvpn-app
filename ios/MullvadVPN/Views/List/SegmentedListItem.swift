import SwiftUI

struct SegmentedListItem: View {
    @State var secondaryButtonHeight: CGFloat = 0

    let title: LocalizedStringKey
    let subtitle: LocalizedStringKey?
    let secondaryButtonImage: Image
    let onSelect: () -> Void
    let onSecondarySelect: () -> Void

    var body: some View {
        HStack(spacing: 2) {
            Button {
                onSelect()
            } label: {
                HStack {
                    VStack {
                        Text(title)
                            .font(.mullvadSmallSemiBold)
                        ifLet(subtitle) { _, value in
                            Text(value)
                                .font(.mullvadMiniSemiBold)
                        }
                    }
                    .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                    .padding(.horizontal, 16)
                    .padding(.vertical, subtitle != nil ? 8 : 16)

                    Spacer()
                }
            }
            .sizeOfView {
                secondaryButtonHeight = $0.height
            }
            .background {
                Color.colorForLevel(0)
            }

            Button {
                onSecondarySelect()
            } label: {
                secondaryButtonImage
            }
            .frame(width: secondaryButtonHeight, height: secondaryButtonHeight)
            .background {
                Color.colorForLevel(0)
            }
            .contentShape(Rectangle())
        }
        .cornerRadius(UIMetrics.LocationList.cellRadius)
    }
}

#Preview {
    @Previewable @State var location: LocationNode = AutomaticLocationNode()

    SegmentedListItem(
        title: "Automatic",
        subtitle: "Sweden, Stockholm",
        secondaryButtonImage: Image.mullvadIconInfo,
        onSelect: {
            print("Selected")
        },
        onSecondarySelect: {
            print("Selected secondary")
        }
    )
}
