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
    func resizeImage(targetSize: CGSize) -> UIImage {
        let widthRatio = targetSize.width / size.width
        let heightRatio = targetSize.height / size.height
        let scaleFactor = min(widthRatio, heightRatio)

        // Calculate new size based on the scale factor
        let newSize = CGSize(width: size.width * scaleFactor, height: size.height * scaleFactor)
        let renderer = UIGraphicsImageRenderer(size: newSize)

        // Render the new image
        let resizedImage = renderer.image { _ in
            draw(in: CGRect(origin: .zero, size: newSize))
        }

        return resizedImage.withRenderingMode(renderingMode)
    }

    func withAlpha(_ alpha: CGFloat) -> UIImage? {
        return UIGraphicsImageRenderer(size: size, format: imageRendererFormat).image { _ in
            draw(in: CGRect(origin: .zero, size: size), blendMode: .normal, alpha: alpha)
        }
    }
}
