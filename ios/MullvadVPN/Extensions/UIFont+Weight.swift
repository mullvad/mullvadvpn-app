//
//  UIFont+Weight.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIFont {
    static func preferredFont(forTextStyle style: TextStyle, weight: Weight) -> UIFont {
        let baseDescriptor = UIFontDescriptor.preferredFontDescriptor(withTextStyle: style)
            .addingAttributes([
                .traits: [UIFontDescriptor.TraitKey.weight: weight],
            ])

        let baseFont = UIFont(descriptor: baseDescriptor, size: 0)
        return UIFontMetrics(forTextStyle: style).scaledFont(for: baseFont)
    }

    func withWeight(_ weight: UIFont.Weight) -> UIFont {
        let newDescriptor = fontDescriptor.addingAttributes([
            .traits: [
                UIFontDescriptor.TraitKey.weight: weight,
            ],
        ])
        return UIFont(descriptor: newDescriptor, size: pointSize)
    }
}
