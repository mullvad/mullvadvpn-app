//
//  UIImage+TintColor.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIImage {
    func backport_withTintColor(_ tintColor: UIColor) -> UIImage {
        return backport_withTintColor(tintColor, renderingMode: renderingMode)
    }

    func backport_withTintColor(_ tintColor: UIColor, renderingMode: RenderingMode) -> UIImage {
        withTintColor(tintColor, renderingMode: renderingMode)
    }
}
