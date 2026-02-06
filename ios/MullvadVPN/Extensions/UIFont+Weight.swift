//
//  UIFont+Weight.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIFont {
    static func preferredFont(forTextStyle style: TextStyle, weight: Weight) -> UIFont {
        return .preferredFont(forTextStyle: style).withWeight(weight)
    }

    func withWeight(_ weight: UIFont.Weight) -> UIFont {
        let newDescriptor = fontDescriptor.addingAttributes([
            .traits: [
                UIFontDescriptor.TraitKey.weight: weight
            ]
        ])
        return UIFont(descriptor: newDescriptor, size: pointSize)
    }
}
