//
//  UIColor+Helpers.swift
//  MullvadVPN
//
//  Created by pronebird on 06/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

extension UIColor {

    func lightened(by factor: CGFloat) -> UIColor? {
        return darkened(by: -factor)
    }

    func darkened(by factor: CGFloat) -> UIColor? {
        var r = CGFloat.zero, g = CGFloat.zero, b = CGFloat.zero, a = CGFloat.zero

        if getRed(&r, green: &g, blue: &b, alpha: &a) {
            return UIColor(red: clampColorComponent(r + factor),
                    green: clampColorComponent(g + factor),
                    blue: clampColorComponent(b + factor),
                    alpha: a)
        }

        return nil
    }

}

private func clampColorComponent(_ value: CGFloat) -> CGFloat {
    return min(1.0, max(value, 0.0))
}
