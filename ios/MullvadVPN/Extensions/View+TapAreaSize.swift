// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
