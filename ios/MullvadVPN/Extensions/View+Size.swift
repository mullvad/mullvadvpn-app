//
//  View+Size.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension View {
    /// Measures view size.
    func sizeOfView(_ onSizeChange: @escaping ((CGSize) -> Void)) -> some View {
        return self
            .background {
                GeometryReader { proxy in
                    Color.clear
                        .preference(key: ViewSizeKey.self, value: proxy.size)
                        .onPreferenceChange(ViewSizeKey.self) { size in
                            onSizeChange(size)
                        }
                }
            }
    }
}

private struct ViewSizeKey: PreferenceKey, Sendable {
    nonisolated(unsafe) static var defaultValue: CGSize = .zero

    static func reduce(value: inout CGSize, nextValue: () -> CGSize) {
        value = nextValue()
    }
}
