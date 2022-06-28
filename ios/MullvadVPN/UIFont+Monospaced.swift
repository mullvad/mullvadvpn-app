//
//  UIFont+Monospaced.swift
//  MullvadVPN
//
//  Created by pronebird on 28/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIFont {
    class func backport_monospacedSystemFont(ofSize size: CGFloat, weight: UIFont.Weight) -> UIFont
    {
        if #available(iOS 13, *) {
            return UIFont.monospacedSystemFont(ofSize: size, weight: weight)
        } else {
            let fontDescriptor = UIFontDescriptor(fontAttributes: [
                .name: "Menlo",
                .traits: [
                    UIFontDescriptor.TraitKey.weight: weight
                ]
            ])

            return UIFont(descriptor: fontDescriptor, size: size)
        }
    }
}
