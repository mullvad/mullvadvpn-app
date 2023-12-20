//
//  AppButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// A subclass that implements action buttons used across the app
class AppButton: CustomButton {
    /// Default content insets based on current trait collection.
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
        /// Default blue appearance.
        case `default`

        /// Destructive appearance.
        case danger

        /// Positive appearance suitable for actions that provide security.
        case success

        /// Translucent destructive appearance.
        case translucentDanger

        /// Translucent neutral appearance.
        case translucentNeutral

        /// Translucent destructive appearance for the left-hand side of a two component split button.
        case translucentDangerSplitLeft

        /// Translucent destructive appearance for the right-hand side of a two component split button.
        case translucentDangerSplitRight

        /// Default blue rounded button suitable for presentation in table view using `.insetGrouped` style.
        case tableInsetGroupedDefault

        /// Positive appearance for presentation in table view using `.insetGrouped` style.
        case tableInsetGroupedSuccess

        /// Destructive style suitable for presentation in table view using `.insetGrouped` style.
        case tableInsetGroupedDanger

        /// Returns a background image for the button.
        var backgroundImage: UIImage {
            switch self {
            case .default:
                UIImage(resource: .defaultButton)
            case .danger:
                UIImage(resource: .dangerButton)
            case .success:
                UIImage(resource: .successButton)
            case .translucentDanger:
                UIImage(resource: .translucentDangerButton)
            case .translucentNeutral:
                UIImage(resource: .translucentNeutralButton)
            case .translucentDangerSplitLeft:
                UIImage(resource: .translucentDangerSplitLeftButton).imageFlippedForRightToLeftLayoutDirection()
            case .translucentDangerSplitRight:
                UIImage(resource: .translucentDangerSplitRightButton).imageFlippedForRightToLeftLayoutDirection()
            case .tableInsetGroupedDefault:
                DynamicAssets.shared.tableInsetGroupedDefaultBackground
            case .tableInsetGroupedSuccess:
                DynamicAssets.shared.tableInsetGroupedSuccessBackground
            case .tableInsetGroupedDanger:
                DynamicAssets.shared.tableInsetGroupedDangerBackground
            }
        }
    }

    /// Button style.
    var style: Style {
        didSet {
            updateButtonBackground()
        }
    }

    /// Prevents updating `contentEdgeInsets` on changes to trait collection.
    var overrideContentEdgeInsets = false

    override var contentEdgeInsets: UIEdgeInsets {
        didSet {
            // Reset inner directional insets when contentEdgeInsets are set directly.
            innerDirectionalContentEdgeInsets = nil
        }
    }

    /// Directional content edge insets that are automatically translated into `contentEdgeInsets` immeditely upon updating the property and on trait collection
    /// changes.
    var directionalContentEdgeInsets: NSDirectionalEdgeInsets {
        get {
            innerDirectionalContentEdgeInsets ?? contentEdgeInsets.toDirectionalInsets
        }
        set {
            innerDirectionalContentEdgeInsets = newValue
            updateContentEdgeInsetsFromDirectional()
        }
    }

    private var innerDirectionalContentEdgeInsets: NSDirectionalEdgeInsets?

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

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func commonInit() {
        super.contentEdgeInsets = defaultContentInsets
        imageAlignment = .trailingFixed

        titleLabel?.font = UIFont.systemFont(ofSize: 18, weight: .semibold)

        let states: [UIControl.State] = [.normal, .highlighted, .disabled]
        states.forEach { state in
            if let titleColor = state.customButtonTitleColor {
                setTitleColor(titleColor, for: state)
            }
        }

        // Avoid setting the background image if it's already set via Interface Builder
        if backgroundImage(for: .normal) == nil {
            updateButtonBackground()
        }
    }

    /// Set background image based on current style.
    private func updateButtonBackground() {
        setBackgroundImage(style.backgroundImage, for: .normal)
    }

    /// Update content edge insets from directional edge insets if set.
    private func updateContentEdgeInsetsFromDirectional() {
        guard let directionalEdgeInsets = innerDirectionalContentEdgeInsets else { return }
        super.contentEdgeInsets = directionalEdgeInsets.toEdgeInsets(effectiveUserInterfaceLayoutDirection)
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if traitCollection.userInterfaceIdiom != previousTraitCollection?.userInterfaceIdiom {
            if overrideContentEdgeInsets {
                updateContentEdgeInsetsFromDirectional()
            } else {
                contentEdgeInsets = defaultContentInsets
            }
        }
    }
}

private extension AppButton {
    class DynamicAssets {
        static let shared = DynamicAssets()

        private init() {}

        /// Default cell corner radius in inset grouped table view
        private let tableViewCellCornerRadius: CGFloat = 10

        lazy var tableInsetGroupedDefaultBackground: UIImage = {
            roundedRectImage(fillColor: .primaryColor)
        }()

        lazy var tableInsetGroupedSuccessBackground: UIImage = {
            roundedRectImage(fillColor: .successColor)
        }()

        lazy var tableInsetGroupedDangerBackground: UIImage = {
            roundedRectImage(fillColor: .dangerColor)
        }()

        private func roundedRectImage(fillColor: UIColor) -> UIImage {
            let cornerRadius = tableViewCellCornerRadius
            let bounds = CGRect(x: 0, y: 0, width: 44, height: 44)
            let image = UIGraphicsImageRenderer(bounds: bounds).image { _ in
                fillColor.setFill()
                UIBezierPath(roundedRect: bounds, cornerRadius: cornerRadius).fill()
            }
            let caps = UIEdgeInsets(top: cornerRadius, left: cornerRadius, bottom: cornerRadius, right: cornerRadius)
            return image.resizableImage(withCapInsets: caps)
        }
    }
}
