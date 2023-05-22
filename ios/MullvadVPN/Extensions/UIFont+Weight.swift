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
        guard let defaultPointSize = style.defaultPointSize else {
            assertionFailure("Could not get default point size from \(style)")
            return preferredFont(forTextStyle: style)
        }

        let metrics = UIFontMetrics(forTextStyle: style)
        let fontToScale = UIFont.systemFont(ofSize: defaultPointSize, weight: weight)

        return metrics.scaledFont(for: fontToScale)
    }
}

private extension UIFont.TextStyle {
    var defaultPointSize: CGFloat? {
        switch self {
        case .largeTitle:
            return 34
        case .title1:
            return 28
        case .title2:
            return 22
        case .title3:
            return 20
        case .headline, .body:
            return 17
        case .callout:
            return 16
        case .subheadline:
            return 15
        case .footnote:
            return 13
        case .caption1:
            return 12
        case .caption2:
            return 11
        default:
            return nil
        }
    }
}
