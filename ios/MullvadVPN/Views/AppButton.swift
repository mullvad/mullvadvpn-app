//
//  AppButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/05/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// A subclass that implements action buttons used across the app
class AppButton: CustomButton {
    /// Default content insets based on current trait collection.
    var defaultContentInsets: NSDirectionalEdgeInsets {
        switch traitCollection.userInterfaceIdiom {
        case .phone:
            return NSDirectionalEdgeInsets(top: 10, leading: 10, bottom: 10, trailing: 10)
        case .pad:
            return NSDirectionalEdgeInsets(top: 15, leading: 15, bottom: 15, trailing: 15)
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
                UIImage(resource: .defaultButton)
            case .tableInsetGroupedSuccess:
                UIImage(resource: .successButton)
            case .tableInsetGroupedDanger:
                UIImage(resource: .dangerButton)
            }
        }
    }

    /// Button style.
    var style: Style {
        didSet {
            updateButtonBackground()
        }
    }

    init(style: Style) {
        self.style = style
        super.init(frame: .zero)
        commonInit()
    }

    override init(frame: CGRect) {
        self.style = .default
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func commonInit() {
        imageAlignment = .trailing
        titleAlignment = .leading

        var config = super.configuration ?? .plain()
        config.title = title(for: state)
        config.contentInsets = defaultContentInsets
        config.background.image = style.backgroundImage
        config.background.imageContentMode = .scaleAspectFill
        config.titleTextAttributesTransformer =
            UIConfigurationTextAttributesTransformer { [weak self] attributeContainer in
                var updatedAttributeContainer = attributeContainer
                updatedAttributeContainer.font = UIFont.systemFont(ofSize: 18, weight: .semibold)
                updatedAttributeContainer.foregroundColor = self?.state.customButtonTitleColor
                return updatedAttributeContainer
            }

        let configurationHandler: UIButton.ConfigurationUpdateHandler = { [weak self] _ in
            guard let self else { return }
            updateButtonBackground()
        }
        configuration = config
        configurationUpdateHandler = configurationHandler
    }

    /// Set background image based on current style.
    private func updateButtonBackground() {
        if isEnabled {
            // Load the normal image and set it as the background
            configuration?.background.image = style.backgroundImage
        } else {
            // Adjust the image for the disabled state
            configuration?.background.image = style.backgroundImage.withAlpha(0.5)
        }
    }
}
