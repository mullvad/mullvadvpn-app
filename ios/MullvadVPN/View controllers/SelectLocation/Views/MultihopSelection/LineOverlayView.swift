import SwiftUI

struct LineOverlayView: View {
    var iconPositions: [AnyHashable: CGRect]
    let isExpanded: Bool

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
        ZStack {
            ForEach(
                iconPositionPairs,
                id: \.0.0
            ) { (curr, next) in
                Line(top: curr.1.bottomCenter, bottom: next.1.topCenter)
                    .stroke(Color.mullvadTextPrimary.opacity(0.6))
                    .opacity(isExpanded ? 1 : 0)
                    .id(curr.0)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .geometryGroup()
    }
}

private struct Line: Shape {
    var top: CGPoint
    var bottom: CGPoint
    private let iconPadding: CGFloat = 2

    var animatableData: AnimatablePair<CGFloat, CGFloat> {
        get {
            AnimatablePair(top.y, bottom.y)
        }
        set {
            top = CGPoint(x: top.x, y: newValue.first)
            bottom = CGPoint(x: bottom.x, y: newValue.second)
        }
    }
    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: top.applying(.init(translationX: 0, y: iconPadding)))
        path.addLine(to: bottom.applying(.init(translationX: 0, y: -iconPadding)))
        return path
    }
}
