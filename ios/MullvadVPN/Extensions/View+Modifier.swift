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
    /**
      A view modifier that can be used to conditionally apply other view modifiers.
     */
    func apply<V: View>(@ViewBuilder _ block: (Self) -> V) -> V { block(self) }

    /**
     Uses the AccessibilityIdentifier you specify to identify the view.
      # Discussion #
     Use this value for testing. It isn’t visible to the user.
     */
    func accessibilityIdentifier(_ id: AccessibilityIdentifier?) -> some View {
        apply {
            if let id {
                $0.accessibilityIdentifier(id.asString)
            } else {
                $0
            }
        }
    }

    /**
     Captures the views frame in the given coordinate system
     */
    func capturePosition(in coordinateSpace: CoordinateSpace, onChange: @escaping (CGRect) -> Void) -> some View {
        background {
            GeometryReader { proxy in
                Color.clear
                    .preference(
                        key: FrameKey.self,
                        value: proxy.frame(in: coordinateSpace)
                    )

            }
            .onPreferenceChange(FrameKey.self) { rect in
                onChange(rect)
            }
        }
    }
}

private struct FrameKey: PreferenceKey {
    static let defaultValue: CGRect = .zero
    static func reduce(value: inout CGRect, nextValue: () -> CGRect) {
        value = nextValue()
    }
}
