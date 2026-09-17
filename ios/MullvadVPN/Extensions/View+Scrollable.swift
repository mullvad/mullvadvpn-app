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

private struct Scrollable: ViewModifier {
    let fillAxes: Axis.Set
    let alignment: Alignment

    func body(content: Content) -> some View {
        GeometryReader { geometry in
            ScrollView {
                AnyView(content)
                    .frame(
                        minWidth: fillAxes.contains(.horizontal) ? geometry.size.width : nil,
                        minHeight: fillAxes.contains(.vertical) ? geometry.size.height : nil,
                        alignment: alignment
                    )
            }
        }.typeErase()
    }
}

extension View {
    func scrollable(fill fillAxes: Axis.Set = [], alignment: Alignment = .center) -> some View {
        modifier(Scrollable(fillAxes: fillAxes, alignment: alignment))
    }
}
