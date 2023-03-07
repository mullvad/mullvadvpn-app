//
//  UIColor+Helpers.swift
//  MullvadVPN
//
//  Created by pronebird on 06/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIColor {
    /// Returns the color lighter by the given percent (in range from 0..1)
    func lightened(by percent: CGFloat) -> UIColor? {
        return darkened(by: -percent)
    }

    /// Returns the color darker by the given percent (in range from 0..1)
    func darkened(by percent: CGFloat) -> UIColor? {
        var r = CGFloat.zero, g = CGFloat.zero, b = CGFloat.zero, a = CGFloat.zero
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
    return min(1.0, max(value, 0.0))
}
