import SwiftUI

@MainActor
extension CoordinateSpace {
    static let multihopSelection: CoordinateSpace = NamedCoordinateSpace.multihopSelection.coordinateSpace
    static let exitLocationScroll: CoordinateSpace = NamedCoordinateSpace.exitLocationScroll.coordinateSpace
}

@MainActor
extension NamedCoordinateSpace {
    static let multihopSelection: NamedCoordinateSpace = .named(
        UUID()
    )
    static let exitLocationScroll: NamedCoordinateSpace = .named(
        UUID()
    )
}
