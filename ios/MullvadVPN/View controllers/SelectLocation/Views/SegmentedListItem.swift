import SwiftUI

struct SegmentedListItem: View {
    @State var infoButtonHeight: CGFloat = 0
    @Binding var location: LocationNode

    let multihopContext: MultihopContext
    let subtitle: LocalizedStringKey?
    let onSelect: (LocationNode) -> Void
    let onInfoSelect: () -> Void

    var body: some View {
        HStack(spacing: 2) {
            Button {
                onSelect(location)
            } label: {
                HStack {
                    VStack {
                        Text(location.name)
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
                infoButtonHeight = $0.height
            }
            .background {
                Color.colorForLevel(0)
            }

            Button {
                onInfoSelect()
            } label: {
                Image.mullvadIconInfo
            }
            .frame(width: infoButtonHeight, height: infoButtonHeight)
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
        location: $location,
        multihopContext: .entry,
        subtitle: nil,
        onSelect: { _ in
            print("Selected node")
        },
        onInfoSelect: {
            print("Selected info on node")
        }
    )
}
