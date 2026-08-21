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
    /// Measures view size.
    func sizeOfView(_ onSizeChange: @escaping ((CGSize) -> Void)) -> some View {
        return
            self
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
