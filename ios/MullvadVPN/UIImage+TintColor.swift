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
        if #available(iOS 13, *) {
            return withTintColor(tintColor, renderingMode: renderingMode)
        }

        let rect = CGRect(origin: .zero, size: size)
        let renderer = UIGraphicsImageRenderer(size: size)

        let renderedImage = renderer.image { context in
            tintColor.setFill()
            context.fill(rect)
            draw(in: rect, blendMode: .destinationIn, alpha: 1)
        }

        return renderedImage.withRenderingMode(renderingMode)
    }
}
