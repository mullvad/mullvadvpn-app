//
//  AppButton.swift
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

private extension UIControl.State {
    var customButtonTitleColor: UIColor? {
        switch self {
        case .normal:
            return UIColor.AppButton.normalTitleColor
        case .disabled:
            return UIColor.AppButton.disabledTitleColor
        case .highlighted:
            return UIColor.AppButton.highlightedTitleColor
        default:
            return nil
        }
    }
}

/// A subclass that implements the button that visually look like URL links on the web
@IBDesignable class LinkButton: CustomButton {

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    override func setTitle(_ title: String?, for state: UIControl.State) {
        if let title = title {
            setAttributedTitle(makeAttributedTitle(title, for: state), for: state)
        } else {
            setAttributedTitle(nil, for: state)
        }
    }

    private func commonInit() {
        imageAlignment = .trailing
        contentHorizontalAlignment = .leading

        let states: [UIControl.State] = [.normal, .highlighted, .disabled]
        states.forEach { (state) in
            if let title = self.title(for: state) {
                let attributedTitle = makeAttributedTitle(title, for: state)
                self.setAttributedTitle(attributedTitle, for: state)
            }
        }
    }

    private func makeAttributedTitle(_ title: String, for state: UIControl.State) -> NSAttributedString {
        var attributes: [NSAttributedString.Key: Any] = [
            .underlineStyle: NSUnderlineStyle.single.rawValue
        ]

        if let titleColor = state.customButtonTitleColor {
            attributes[.foregroundColor] = titleColor
        }

        return NSAttributedString(string: title, attributes: attributes)
    }
}

/// A subclass that implements action buttons used across the app
@IBDesignable class AppButton: CustomButton {

    var defaultContentInsets: UIEdgeInsets {
        switch traitCollection.userInterfaceIdiom {
        case .phone:
            return UIEdgeInsets(top: 10, left: 10, bottom: 10, right: 10)
        case .pad:
            return UIEdgeInsets(top: 15, left: 15, bottom: 15, right: 15)
        default:
            return .zero
        }
    }

    enum Style: Int {
        case `default`
        case danger
        case success
        case translucentDanger
        case translucentNeutral
        case translucentDangerSplitLeft
        case translucentDangerSplitRight

        var backgroundImage: UIImage? {
            switch self {
            case .default:
                return UIImage(named: "DefaultButton")
            case .danger:
                return UIImage(named: "DangerButton")
            case .success:
                return UIImage(named: "SuccessButton")
            case .translucentDanger:
                return UIImage(named: "TranslucentDangerButton")
            case .translucentNeutral:
                return UIImage(named: "TranslucentNeutralButton")
            case .translucentDangerSplitLeft:
                return UIImage(named: "TranslucentDangerSplitLeftButton")
            case .translucentDangerSplitRight:
                return UIImage(named: "TranslucentDangerSplitRightButton")
            }
        }
    }

    var style: Style {
        didSet {
            updateButtonBackground()
        }
    }

    @IBInspectable var interfaceBuilderStyle: Int {
        get {
            return self.style.rawValue
        }
        set {
            if let style = Style(rawValue: newValue) {
                self.style = style
            }
        }
    }

    var overrideContentEdgeInsets = false

    init(style: Style) {
        self.style = style
        super.init(frame: .zero)
        commonInit()
    }

    override init(frame: CGRect) {
        style = .default
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder aDecoder: NSCoder) {
        style = .default
        super.init(coder: aDecoder)
        commonInit()
    }

    private func commonInit() {
        var contentInsets = contentEdgeInsets

        if contentInsets.top == 0 {
            contentInsets.top = self.defaultContentInsets.top
        }

        if contentInsets.bottom == 0 {
            contentInsets.bottom = self.defaultContentInsets.bottom
        }

        if contentInsets.right == 0 {
            contentInsets.right = self.defaultContentInsets.right
        }

        if contentInsets.left == 0 {
            contentInsets.left = self.defaultContentInsets.left
        }

        contentEdgeInsets = contentInsets
        imageAlignment = .trailingFixed

        titleLabel?.font = UIFont.systemFont(ofSize: 17, weight: .semibold)

        let states: [UIControl.State] = [.normal, .highlighted, .disabled]
        states.forEach { (state) in
            if let titleColor = state.customButtonTitleColor {
                setTitleColor(titleColor, for: state)
            }
        }

        // Avoid setting the background image if it's already set via Interface Builder
        if backgroundImage(for: .normal) == nil {
            updateButtonBackground()
        }
    }

    private func updateButtonBackground() {
        setBackgroundImage(style.backgroundImage, for: .normal)
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if traitCollection.userInterfaceIdiom != previousTraitCollection?.userInterfaceIdiom, !overrideContentEdgeInsets {
            contentEdgeInsets = self.defaultContentInsets
        }
    }

}

/// A custom `UIButton` subclass that implements additional layouts for the image
@IBDesignable class CustomButton: UIButton {

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

    override func layoutSubviews() {
        super.layoutSubviews()

        if #available(iOS 13, *) {
            // no-op
        } else {
            // Fix: on iOS 12 the image view frame is not always set, even though the `UIButton`
            // calls `imageRect` to compute the image layout frame.
            let imageRect = self.imageRect(forContentRect: contentRect(forBounds: bounds))
            if imageView?.frame != imageRect {
                imageView?.frame = imageRect
            }
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

        default:
            fatalError()
        }

        return (titleRect, imageRect)
    }

    override func imageRect(forContentRect contentRect: CGRect) -> CGRect {
        return computeLayout(forContentRect: contentRect).1
    }

    override func titleRect(forContentRect contentRect: CGRect) -> CGRect {
        return computeLayout(forContentRect: contentRect).0
    }

}
