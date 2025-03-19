//
//  UIImage+Helpers.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-10-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIImage {
    // Function to resize image while keeping aspect ratio
    // if `trimmingBorder` is specified, that number of pixels will be trimmed off each side before the remaining area is rendered to the new image
    func resized(to: CGSize, trimmingBorder border: CGFloat = 0) -> UIImage {
        let sourceSize = CGSize(width: size.width - 2 * border, height: size.height - 2 * border)
        let widthRatio = to.width / sourceSize.width
        let heightRatio = to.height / sourceSize.height
        let scaleFactor = min(widthRatio, heightRatio)
        let scaledBorder = border * scaleFactor

        // Calculate new size based on the scale factor
        let newSize = CGSize(width: sourceSize.width * scaleFactor, height: sourceSize.height * scaleFactor)
        let renderer = UIGraphicsImageRenderer(size: newSize)

        // Render the new image
        let resizedImage = renderer.image { _ in
            draw(
                in: CGRect(
                    origin: .init(x: -scaledBorder, y: -scaledBorder),
                    size: .init(width: newSize.width + 2 * scaledBorder, height: newSize.height + 2 * scaledBorder)
                )
            )
        }

        return resizedImage.withRenderingMode(renderingMode)
    }

    func withAlpha(_ alpha: CGFloat) -> UIImage? {
        return UIGraphicsImageRenderer(size: size, format: imageRendererFormat).image { _ in
            draw(in: CGRect(origin: .zero, size: size), blendMode: .normal, alpha: alpha)
        }
    }
}
