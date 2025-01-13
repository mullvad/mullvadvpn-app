//
//  View+TapAreaSize.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension View {
    /// Adjusts tappable area to at least minimum (default) size without changing
    /// actual view size.
    func adjustingTapAreaSize() -> some View {
        modifier(TappablePadding())
    }
}

private struct TappablePadding: ViewModifier {
    @State private var actualViewSize: CGSize = .zero
    private let tappableViewSize = UIMetrics.Button.minimumTappableAreaSize

    func body(content: Content) -> some View {
        content
            .sizeOfView { actualViewSize = $0 }
            .frame(
                width: max(actualViewSize.width, tappableViewSize.width),
                height: max(actualViewSize.height, tappableViewSize.height)
            )
            .contentShape(Rectangle())
    }
}
