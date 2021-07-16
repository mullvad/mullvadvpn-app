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
    let logoImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "LogoIcon"))
        imageView.translatesAutoresizingMaskIntoConstraints = false
        return imageView
    }()

    lazy var titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.text = "MULLVAD VPN"
        titleLabel.font = UIFont.boldSystemFont(ofSize: 24)
        titleLabel.textColor = UIColor.white.withAlphaComponent(0.8)
        titleLabel.accessibilityTraits.insert(.header)
        return titleLabel
    }()

    let settingsButton = makeSettingsButton()

    class func makeSettingsButton() -> UIButton {
        let settingsButton = UIButton(type: .custom)
        settingsButton.setImage(UIImage(named: "IconSettings"), for: .normal)
        settingsButton.translatesAutoresizingMaskIntoConstraints = false
        settingsButton.accessibilityIdentifier = "SettingsButton"
        settingsButton.accessibilityLabel = NSLocalizedString("HEADER_BAR_SETTINGS_BUTTON_ACCESSIBILITY_LABEL", comment: "")
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

        if #available(iOS 13.0, *) {
            accessibilityContainerType = .semanticGroup
        }

        let constraints = [
            logoImageView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            logoImageView.centerYAnchor.constraint(equalTo: titleLabel.centerYAnchor),
            logoImageView.widthAnchor.constraint(equalToConstant: 44),
            logoImageView.heightAnchor.constraint(equalTo: logoImageView.widthAnchor, multiplier: 1),

            titleLabel.leadingAnchor.constraint(equalTo: logoImageView.trailingAnchor, constant: 8),
            titleLabel.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor, constant: 22),
            layoutMarginsGuide.bottomAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 22),

            settingsButton.leadingAnchor.constraint(greaterThanOrEqualTo: titleLabel.trailingAnchor, constant: 8),
            settingsButton.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            settingsButton.centerYAnchor.constraint(equalTo: titleLabel.centerYAnchor)
        ]

        [logoImageView, titleLabel, settingsButton].forEach { addSubview($0) }

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
