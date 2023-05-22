//
//  UIFont+Weight.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-23.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIFont {
    static func preferredFont(forTextStyle style: TextStyle, weight: Weight) -> UIFont {
        let descriptor = UIFontDescriptor.preferredFontDescriptor(withTextStyle: style)
            .addingAttributes([
                .traits: [UIFontDescriptor.TraitKey.weight: weight],
            ])

        return UIFont(descriptor: descriptor, size: 0)
    }
}
