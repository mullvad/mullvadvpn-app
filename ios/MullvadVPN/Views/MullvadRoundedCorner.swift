import SwiftUI

/*
 This is an animatable version of RoundedRectangle. It can also round only a subset of corners.
 */
struct MullvadRoundedCorner: Shape {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insetBy: CGFloat = 0

    private var radii: CornerRadii {
        CornerRadii(
            topLeft: corners.contains(.topLeft) ? cornerRadius : 0,
            topRight: corners.contains(.topRight) ? cornerRadius : 0,
            bottomLeft: corners.contains(.bottomLeft) ? cornerRadius : 0,
            bottomRight: corners.contains(.bottomRight) ? cornerRadius : 0
        )
    }

    var animatableData: CornerRadii {
        get { radiiState }
        set { radiiState = newValue }
    }

    private var radiiState: CornerRadii

    init(
        cornerRadius: CGFloat = .infinity,
        corners: UIRectCorner = .allCorners,
        insetBy: CGFloat = 0
    ) {
        self.cornerRadius = cornerRadius
        self.corners = corners
        self.insetBy = insetBy
        self.radiiState = CornerRadii(
            topLeft: corners.contains(.topLeft) ? cornerRadius : 0,
            topRight: corners.contains(.topRight) ? cornerRadius : 0,
            bottomLeft: corners.contains(.bottomLeft) ? cornerRadius : 0,
            bottomRight: corners.contains(.bottomRight) ? cornerRadius : 0
        )
    }

    func path(in rect: CGRect) -> Path {
        let insetRect = rect.insetBy(dx: insetBy, dy: insetBy)
        var path = Path()

        let tl = min(radiiState.topLeft, min(insetRect.width, insetRect.height) / 2)
        let tr = min(radiiState.topRight, min(insetRect.width, insetRect.height) / 2)
        let bl = min(radiiState.bottomLeft, min(insetRect.width, insetRect.height) / 2)
        let br = min(radiiState.bottomRight, min(insetRect.width, insetRect.height) / 2)

        path.move(to: CGPoint(x: insetRect.minX + tl, y: insetRect.minY))

        // Top edge
        path.addLine(to: CGPoint(x: insetRect.maxX - tr, y: insetRect.minY))
        path.addArc(
            center: CGPoint(x: insetRect.maxX - tr, y: insetRect.minY + tr),
            radius: tr,
            startAngle: .degrees(-90),
            endAngle: .degrees(0),
            clockwise: false
        )

        // Right edge
        path.addLine(to: CGPoint(x: insetRect.maxX, y: insetRect.maxY - br))
        path.addArc(
            center: CGPoint(x: insetRect.maxX - br, y: insetRect.maxY - br),
            radius: br,
            startAngle: .degrees(0),
            endAngle: .degrees(90),
            clockwise: false
        )

        // Bottom edge
        path.addLine(to: CGPoint(x: insetRect.minX + bl, y: insetRect.maxY))
        path.addArc(
            center: CGPoint(x: insetRect.minX + bl, y: insetRect.maxY - bl),
            radius: bl,
            startAngle: .degrees(90),
            endAngle: .degrees(180),
            clockwise: false
        )

        // Left edge
        path.addLine(to: CGPoint(x: insetRect.minX, y: insetRect.minY + tl))
        path.addArc(
            center: CGPoint(x: insetRect.minX + tl, y: insetRect.minY + tl),
            radius: tl,
            startAngle: .degrees(180),
            endAngle: .degrees(270),
            clockwise: false
        )

        return path
    }

    struct CornerRadii: VectorArithmetic {
        var topLeft: CGFloat
        var topRight: CGFloat
        var bottomLeft: CGFloat
        var bottomRight: CGFloat

        static let zero = CornerRadii(
            topLeft: 0,
            topRight: 0,
            bottomLeft: 0,
            bottomRight: 0
        )

        static func + (lhs: CornerRadii, rhs: CornerRadii) -> CornerRadii {
            .init(
                topLeft: lhs.topLeft + rhs.topLeft,
                topRight: lhs.topRight + rhs.topRight,
                bottomLeft: lhs.bottomLeft + rhs.bottomLeft,
                bottomRight: lhs.bottomRight + rhs.bottomRight
            )
        }

        static func - (lhs: CornerRadii, rhs: CornerRadii) -> CornerRadii {
            .init(
                topLeft: lhs.topLeft - rhs.topLeft,
                topRight: lhs.topRight - rhs.topRight,
                bottomLeft: lhs.bottomLeft - rhs.bottomLeft,
                bottomRight: lhs.bottomRight - rhs.bottomRight
            )
        }

        mutating func scale(by rhs: Double) {
            topLeft *= rhs
            topRight *= rhs
            bottomLeft *= rhs
            bottomRight *= rhs
        }

        var magnitudeSquared: Double {
            Double(
                topLeft * topLeft + topRight * topRight + bottomLeft * bottomLeft + bottomRight
                    * bottomRight
            )
        }

        static func == (lhs: CornerRadii, rhs: CornerRadii) -> Bool {
            lhs.topLeft == rhs.topLeft && lhs.topRight == rhs.topRight && lhs.bottomLeft == rhs.bottomLeft
                && lhs.bottomRight == rhs.bottomRight
        }

        // swiftlint:disable:next shorthand_operator
        mutating func add(_ other: CornerRadii) { self = self + other }
        // swiftlint:disable:next shorthand_operator
        mutating func subtract(_ other: CornerRadii) { self = self - other }
    }
}
