import SwiftUI

struct SegmentedListItemOLD: View {
    @State var secondaryButtonHeight: CGFloat = UIMetrics.LocationList.cellMinHeight

    let title: LocalizedStringKey
    let subtitle: LocalizedStringKey?
    let onSelect: () -> Void
    var minHeight: CGFloat?
    var leadingView: AnyView?
    var secondaryButtonImage: Image?
    var onSecondarySelect: (() -> Void)?

    var body: some View {
        HStack(spacing: 2) {
            Button {
                onSelect()
            } label: {
                HStack {
                    leadingView

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
                    //                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .frame(minHeight: minHeight)

                    Spacer()
                }
                //                .background {
                //                    Color.colorForLevel(0)
                //                }
            }
            .sizeOfView {
                secondaryButtonHeight = $0.height
            }

            //            if let secondaryButtonImage {
            //                Button {
            //                    onSecondarySelect?()
            //                } label: {
            //                    secondaryButtonImage
            //                        .frame(width: minHeight, height: secondaryButtonHeight)
            //                        .background {
            //                            Color.colorForLevel(0)
            //                        }
            //                }
            //                .contentShape(Rectangle())
            //            }
        }
        //        .clipShape(RoundedRectangle(cornerRadius: UIMetrics.LocationList.cellCornerRadius))
    }
}

#Preview {
    SegmentedListItemOLD(
        title: "Automatic",
        subtitle: "Sweden, Stockholm",
        onSelect: {
            print("Selected")
        },
        secondaryButtonImage: Image.mullvadIconInfo,
        onSecondarySelect: {
            print("Selected secondary")
        }
    )
}
