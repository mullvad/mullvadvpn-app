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

struct RoundedCornerModifier: ViewModifier {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insertBy: CGFloat = 0

    var borderColor: Color = .clear
    var borderWidth: CGFloat = 0

    func body(content: Content) -> some View {
        content
            .clipShape(
                RoundedCorner(
                    cornerRadius: cornerRadius,
                    corners: corners,
                    insertBy: insertBy
                )
            )
            .overlay {
                RoundedCorner(
                    cornerRadius: cornerRadius,
                    corners: corners,
                    insertBy: insertBy
                )
                .stroke(borderColor, lineWidth: borderWidth)
            }
    }
}

private struct RoundedCorner: Shape {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insertBy: CGFloat = 0

    func path(in rect: CGRect) -> Path {
        let insetRect = rect.insetBy(dx: insertBy, dy: insertBy)
        let path = UIBezierPath(
            roundedRect: insetRect,
            byRoundingCorners: corners,
            cornerRadii: CGSize(width: cornerRadius, height: cornerRadius)
        )
        return Path(path.cgPath)
    }
}
