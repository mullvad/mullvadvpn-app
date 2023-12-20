//
//  CustomButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum ButtonImageAlignment {
    /// Align image at the left edge of the title label
    case left

    /// Align image at the right edge of the title label
    case right

    /// Align image at the leading edge of the title label
    case leading

    /// Align image at the trailing edge of the title label
    case trailing

    /// Align image at the leading edge of content area
    case leadingFixed

    /// Align image at the trailing edge of the content area
    case trailingFixed

    /// Align image at the left edge of the content area
    case leftFixed

    /// Align image at the right edge of the content area
    case rightFixed
}

extension UIControl.State {
    var customButtonTitleColor: UIColor? {
        switch self {
        case .normal:
            return UIColor.AppButton.normalTitleColor
        case .disabled:
            return UIColor.AppButton.disabledTitleColor.withAlphaComponent(0.5)
        case .highlighted:
            return UIColor.AppButton.highlightedTitleColor
        default:
            return nil
        }
    }
}

/// A custom `UIButton` subclass that implements additional layouts for the image
class CustomButton: UIButton {
    var imageAlignment: ButtonImageAlignment = .leading {
        didSet {
            invalidateIntrinsicContentSize()
        }
    }

    var inlineImageSpacing: CGFloat = 4 {
        didSet {
            invalidateIntrinsicContentSize()
        }
    }

    override var intrinsicContentSize: CGSize {
        var intrinsicSize = super.intrinsicContentSize

        // Add spacing between the image and title label in intrinsic size calculation
        if let imageSize = currentImage?.size, imageSize.width > 0 {
            intrinsicSize.width += inlineImageSpacing
        }

        return intrinsicSize
    }

    var effectiveImageAlignment: ButtonImageAlignment {
        switch (imageAlignment, effectiveUserInterfaceLayoutDirection) {
        case (.left, _),
             (.leading, .leftToRight),
             (.trailing, .rightToLeft):
            return .left

        case (.right, _),
             (.trailing, .leftToRight),
             (.leading, .rightToLeft):
            return .right

        case (.leftFixed, _),
             (.leadingFixed, .leftToRight),
             (.trailingFixed, .rightToLeft):
            return .leftFixed

        case (.rightFixed, _),
             (.trailingFixed, .leftToRight),
             (.leadingFixed, .rightToLeft):
            return .rightFixed

        default:
            fatalError()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    private func commonInit() {
        // Align the text color with the tint color which is applied to the image view
        if let imageTintColor = UIControl.State.normal.customButtonTitleColor {
            tintColor = imageTintColor
        }
    }

    private func computeLayout(forContentRect contentRect: CGRect) -> (CGRect, CGRect) {
        var imageRect = super.imageRect(forContentRect: contentRect)
        var titleRect = super.titleRect(forContentRect: contentRect)

        switch (effectiveContentHorizontalAlignment, effectiveImageAlignment) {
        case (.left, .left):
            imageRect.origin.x = contentRect.minX
            titleRect.origin.x = imageRect.width > 0
                ? imageRect.maxX + inlineImageSpacing
                : contentRect.minX

        case (.left, .right):
            titleRect.origin.x = contentRect.minX
            imageRect.origin.x = titleRect.maxX + inlineImageSpacing

        case (.left, .leftFixed):
            imageRect.origin.x = contentRect.minX
            titleRect.origin.x = imageRect.width > 0
                ? imageRect.maxX + inlineImageSpacing
                : contentRect.minX

        case (.left, .rightFixed):
            imageRect.origin.x = contentRect.maxX - imageRect.width
            titleRect.origin.x = contentRect.minX

        case (.center, .leftFixed):
            imageRect.origin.x = contentRect.minX
            titleRect.origin.x = contentRect.midX - titleRect.width * 0.5

        case (.center, .rightFixed):
            imageRect.origin.x = contentRect.maxX - imageRect.width
            titleRect.origin.x = contentRect.midX - titleRect.width * 0.5

        case (.center, .left):
            titleRect.origin.x = contentRect.midX - titleRect.width * 0.5
            imageRect.origin.x = titleRect.minX - inlineImageSpacing - imageRect.width

        case (.center, .right):
            titleRect.origin.x = contentRect.midX - titleRect.width * 0.5
            imageRect.origin.x = titleRect.maxX + inlineImageSpacing

        case (.right, .left):
            titleRect.origin.x = contentRect.maxX - titleRect.width
            imageRect.origin.x = titleRect.minX - imageRect.width - inlineImageSpacing

        case (.right, .leftFixed):
            imageRect.origin.x = contentRect.minX
            titleRect.origin.x = contentRect.maxX - titleRect.width

        case (.right, .rightFixed):
            imageRect.origin.x = contentRect.maxX - imageRect.width
            titleRect.origin.x = imageRect.width > 0
                ? imageRect.minX - inlineImageSpacing - titleRect.width
                : contentRect.maxX - titleRect.width

        default:
            fatalError()
        }

        return (titleRect, imageRect)
    }

    override func imageRect(forContentRect contentRect: CGRect) -> CGRect {
        computeLayout(forContentRect: contentRect).1
    }

    override func titleRect(forContentRect contentRect: CGRect) -> CGRect {
        computeLayout(forContentRect: contentRect).0
    }
}
