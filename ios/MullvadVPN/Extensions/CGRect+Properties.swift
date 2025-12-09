import Foundation

extension CGRect {
    var center: CGPoint {
        CGPoint(x: midX, y: midY)
    }

    var topCenter: CGPoint {
        CGPoint(x: midX, y: minY)
    }

    var bottomCenter: CGPoint {
        CGPoint(x: midX, y: maxY)
    }
}
