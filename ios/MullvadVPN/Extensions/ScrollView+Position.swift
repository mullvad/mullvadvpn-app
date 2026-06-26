//
//  ScrollView+Position.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-06-26.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension CoordinateSpace {
    static let scroll = "scroll"
}

extension View {
    /// Measures view position in a `ScrollView`.
    func scrollPosition(_ onPositionChange: @escaping ((CGFloat) -> Void)) -> some View {
        self
            .background {
                GeometryReader { proxy in
                    Color.clear
                        .preference(
                            key: ViewOffsetKey.self,
                            value: proxy.frame(in: .named(CoordinateSpace.scroll)).origin.y
                        )
                        .onPreferenceChange(ViewOffsetKey.self) { size in
                            onPositionChange(size)
                        }
                }
            }
    }
}

private struct ViewOffsetKey: PreferenceKey, Sendable {
    nonisolated(unsafe) static var defaultValue: CGFloat = .zero

    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}
