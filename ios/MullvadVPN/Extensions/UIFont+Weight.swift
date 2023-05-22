//
//  UIFont+Weight.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-23.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIFont {
    static func preferredFont(forTextStyle style: UIFont.TextStyle, weight: Weight) -> UIFont {
        let metrics = UIFontMetrics(forTextStyle: style)
        let fontToScale = UIFont.systemFont(ofSize: style.defaultPointSize, weight: weight)

        return metrics.scaledFont(for: fontToScale)
    }
}

private extension UIFont.TextStyle {
    var defaultPointSize: CGFloat {
        let traitCollection = UITraitCollection(
            preferredContentSizeCategory: UIApplication.shared
                .preferredContentSizeCategory
        )
        let font = UIFont.preferredFont(forTextStyle: self, compatibleWith: traitCollection)

        return font.pointSize
    }
}
