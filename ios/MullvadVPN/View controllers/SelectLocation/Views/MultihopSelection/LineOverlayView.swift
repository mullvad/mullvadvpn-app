import SwiftUI

struct LineOverlayView: View {
    var iconPositions: [AnyHashable: CGRect]
    let isExpanded: Bool
    private let iconPadding: CGFloat = 2
    private let lineWidth: CGFloat = 1
    // icon position pairs from top to bottom
    var iconPositionPairs: [((AnyHashable, CGRect), (AnyHashable, CGRect))] {
        let sorted =
            iconPositions
            .sorted {
                $0.1.origin.y < $1.1.origin.y
            }
        return zip(sorted, sorted.dropFirst())
            .map { curr, next in
                ((curr.key, curr.value), (next.key, next.value))
            }
    }
    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(
                iconPositionPairs,
                id: \.0.0
            ) { (curr, next) in
                let length = max(next.1.topCenter.y - curr.1.bottomCenter.y - 2 * iconPadding, 0)
                Color.mullvadTextPrimary.opacity(0.6)
                    .frame(
                        width: lineWidth,
                        height: length
                    )
                    .padding(.top, curr.1.bottomCenter.y + iconPadding)
                    .padding(.leading, curr.1.midX - lineWidth / 2)
                    .opacity(isExpanded ? 1 : 0)
                    .animation(.default, value: curr.1.midX)
                    .id(curr.0)
                    .transition(
                        .asymmetric(
                            insertion: .opacity.animation(.default.delay(0.2)),
                            removal: .identity
                        )
                    )
            }
        }
        .animation(.default, value: iconPositionPairs.count)
    }
}
