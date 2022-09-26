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
        UIFont.monospacedSystemFont(ofSize: size, weight: weight)
    }
}
