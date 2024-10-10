//
//  UIImage+Helpers.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-10-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIImage {
    // Function to resize image while keeping aspect ratio
    func resizeImage(targetSize: CGSize) -> UIImage {
        let widthRatio = targetSize.width / self.size.width
        let heightRatio = targetSize.height / self.size.height
        let scaleFactor = min(widthRatio, heightRatio)

        // Calculate new size based on the scale factor
        let newSize = CGSize(width: self.size.width * scaleFactor, height: self.size.height * scaleFactor)
        let renderer = UIGraphicsImageRenderer(size: newSize)

        // Render the new image
        let resizedImage = renderer.image { _ in
            self.draw(in: CGRect(origin: .zero, size: newSize))
        }

        return resizedImage.withRenderingMode(self.renderingMode)
    }
}
