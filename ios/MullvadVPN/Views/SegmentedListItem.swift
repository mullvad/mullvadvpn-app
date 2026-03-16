import SwiftUI

struct SegmentedListItem: View {
    @State var secondaryButtonHeight: CGFloat = UIMetrics.LocationList.cellMinHeight

    let title: LocalizedStringKey
    let subtitle: LocalizedStringKey?
    let secondaryButtonImage: Image
    let onSelect: () -> Void
    let onSecondarySelect: () -> Void
    var minHeight: CGFloat?

    var body: some View {
        HStack(spacing: 2) {
            Button {
                onSelect()
            } label: {
                HStack {
                    VStack {
                        Text(title)
                            .font(.mullvadSmallSemiBold)
                            .foregroundStyle(Color.mullvadTextPrimary)
                        if let subtitle {
                            Text(subtitle)
                                .font(.mullvadMiniSemiBold)
                                .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .frame(minHeight: minHeight ?? 0)

                    Spacer()
                }
                .background {
                    Color.colorForLevel(0)
                }
            }
            .sizeOfView {
                secondaryButtonHeight = $0.height
            }

            Button {
                onSecondarySelect()
            } label: {
                secondaryButtonImage
                    .frame(width: secondaryButtonHeight, height: secondaryButtonHeight)
                    .background {
                        Color.colorForLevel(0)
                    }
            }
            .contentShape(Rectangle())
        }
        .cornerRadius(UIMetrics.LocationList.cellCornerRadius)
    }
}

#Preview {
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
