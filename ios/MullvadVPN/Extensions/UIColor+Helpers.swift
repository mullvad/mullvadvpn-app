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

extension UIColor {
    var color: Color {
        Color(self)
    }

    /// Returns the color lighter by the given percent (in range from 0..1)
    func lightened(by percent: CGFloat) -> UIColor? {
        darkened(by: -percent)
    }

    /// Returns the color darker by the given percent (in range from 0..1)
    func darkened(by percent: CGFloat) -> UIColor? {
        var r = CGFloat.zero
        var g = CGFloat.zero
        var b = CGFloat.zero
        var a = CGFloat.zero
        let factor = 1.0 - percent

        if getRed(&r, green: &g, blue: &b, alpha: &a) {
            return UIColor(
                red: clampColorComponent(r * factor),
                green: clampColorComponent(g * factor),
                blue: clampColorComponent(b * factor),
                alpha: a
            )
        }

        return nil
    }
}

private func clampColorComponent(_ value: CGFloat) -> CGFloat {
    min(1.0, max(value, 0.0))
}
