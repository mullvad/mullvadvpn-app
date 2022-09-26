//
//  HeaderBarView.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class HeaderBarView: UIView {
    private let brandNameImage = UIImage(named: "LogoText")?
        .withTintColor(UIColor.HeaderBar.brandNameColor, renderingMode: .alwaysOriginal)

    let logoImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "LogoIcon"))
        imageView.translatesAutoresizingMaskIntoConstraints = false
        return imageView
    }()

    lazy var brandNameImageView: UIImageView = {
        let imageView = UIImageView(image: brandNameImage)
        imageView.translatesAutoresizingMaskIntoConstraints = false
        imageView.contentMode = .scaleAspectFill
        return imageView
    }()

    let settingsButton = makeSettingsButton()

    class func makeSettingsButton() -> HeaderBarButton {
        let settingsImage = UIImage(named: "IconSettings")?
            .withTintColor(UIColor.HeaderBar.buttonColor, renderingMode: .alwaysOriginal)
        let disabledSettingsImage = UIImage(named: "IconSettings")?
            .withTintColor(
                UIColor.HeaderBar.disabledButtonColor,
                renderingMode: .alwaysOriginal
            )

        let settingsButton = HeaderBarButton(type: .system)
        settingsButton.setImage(settingsImage, for: .normal)
        settingsButton.setImage(disabledSettingsImage, for: .disabled)
        settingsButton.translatesAutoresizingMaskIntoConstraints = false
        settingsButton.accessibilityIdentifier = "SettingsButton"
        settingsButton.accessibilityLabel = NSLocalizedString(
            "HEADER_BAR_SETTINGS_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "HeaderBar",
            value: "Settings",
            comment: ""
        )
        return settingsButton
    }

    private let borderLayer: CALayer = {
        let layer = CALayer()
        layer.backgroundColor = UIColor.HeaderBar.dividerColor.cgColor
        return layer
    }()

    var showsDivider = false {
        didSet {
            if showsDivider {
                layer.addSublayer(borderLayer)
            } else {
                borderLayer.removeFromSuperlayer()
            }
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        layoutMargins = UIEdgeInsets(
            top: 0,
            left: UIMetrics.contentLayoutMargins.left,
            bottom: 0,
            right: UIMetrics.contentLayoutMargins.right
        )

        accessibilityContainerType = .semanticGroup

        let imageSize = brandNameImage?.size ?? .zero
        let brandNameAspectRatio = imageSize.width / max(imageSize.height, 1)

        let constraints = [
            logoImageView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            logoImageView.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor),
            logoImageView.widthAnchor.constraint(equalToConstant: 44),
            logoImageView.heightAnchor.constraint(
                equalTo: logoImageView.widthAnchor,
                multiplier: 1
            ),

            brandNameImageView.leadingAnchor.constraint(
                equalTo: logoImageView.trailingAnchor,
                constant: 9
            ),
            brandNameImageView.topAnchor.constraint(
                equalTo: layoutMarginsGuide.topAnchor,
                constant: 22
            ),
            brandNameImageView.widthAnchor.constraint(
                equalTo: brandNameImageView.heightAnchor,
                multiplier: brandNameAspectRatio
            ),
            brandNameImageView.heightAnchor.constraint(equalToConstant: 18),
            layoutMarginsGuide.bottomAnchor.constraint(
                equalTo: brandNameImageView.bottomAnchor,
                constant: 22
            ),

            settingsButton.leadingAnchor.constraint(
                greaterThanOrEqualTo: brandNameImageView.trailingAnchor,
                constant: 8
            ),
            settingsButton.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            settingsButton.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor),
        ]

        [logoImageView, brandNameImageView, settingsButton].forEach { addSubview($0) }

        NSLayoutConstraint.activate(constraints)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        borderLayer.frame = CGRect(x: 0, y: frame.maxY - 1, width: frame.width, height: 1)
    }
}
