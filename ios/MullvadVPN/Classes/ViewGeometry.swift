//
//  ViewGeometry.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

/// Struct for measuring view size. Typically used in a `.background()`,
/// eg. `.background(ViewGeometry { actualViewSize in [...] }).
struct ViewGeometry: View {
    let onSizeChange: (CGSize) -> Void

    var body: some View {
        GeometryReader { geometry in
            Color.clear
                .preference(key: ViewSizeKey.self, value: geometry.size)
                .onPreferenceChange(ViewSizeKey.self) {
                    onSizeChange($0)
                }
        }
    }
}

private struct ViewSizeKey: PreferenceKey {
    static var defaultValue: CGSize = .zero

    static func reduce(value: inout CGSize, nextValue: () -> CGSize) {
        value = nextValue()
    }
}
