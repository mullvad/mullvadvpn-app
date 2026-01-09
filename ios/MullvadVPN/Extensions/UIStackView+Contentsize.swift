//
//  UIStackView+Contentsize.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import UIKit

extension UIStackView {
    var isOverflowed: Bool {
        return contentSize.width > bounds.width
    }
    
    var contentSize: CGSize {
        layoutIfNeeded()
        let availableSize = bounds.size
        guard availableSize.width > 0 else { return availableSize }
        return systemLayoutSizeFitting(
            CGSize(
                width: CGFloat.greatestFiniteMagnitude,
                height: availableSize.height),
            withHorizontalFittingPriority: .fittingSizeLevel,
            verticalFittingPriority: .fittingSizeLevel
        )
    }
}
