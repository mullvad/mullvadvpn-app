import SwiftUI

struct HopView: View {
    let hop: Hop
    let isSelected: Bool
    let onFilterTapped: () -> Void
    let onIconPositionChange: (CGRect) -> Void
    var body: some View {
        HStack {
            hop.icon
                .capturePosition(
                    in: .multihopSelection
                ) { position in
                    onIconPositionChange(position)
                }
            Text(hop.selectedLocation?.name ?? "Select location")
                .lineLimit(nil)
                .fixedSize(horizontal: false, vertical: true)
            Spacer()
            // TODO: use individual filter buttons when settings have been migrated
            //            Button {
            //                onFilterTapped()
            //            } label: {
            //                Image(systemName: "line.3.horizontal.decrease")
            //            }
        }
        .font(.mullvadSmallSemiBold)
        .foregroundStyle(
            isSelected
                ? Color.mullvadTextPrimary
                : Color.mullvadTextPrimary
                    .opacity(0.6)
        )
        .padding(8)
    }
}
