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

    private let logoImageView = UIImageView(image: UIImage(named: "LogoIcon"))

    private lazy var brandNameImageView: UIImageView = {
        let imageView = UIImageView(image: brandNameImage)
        imageView.contentMode = .scaleAspectFill
        return imageView
    }()

    private let deviceInfoHolder: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.spacing = 16.0
        return stackView
    }()

    private lazy var deviceName: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.8)
        label.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return label
    }()

    private lazy var timeLeft: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.8)
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        return label
    }()

    private lazy var buttonContainer: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [accountButton, settingsButton])
        stackView.spacing = UIMetrics.headerBarButtonSpacing
        return stackView
    }()

    private let borderLayer: CALayer = {
        let layer = CALayer()
        layer.backgroundColor = UIColor.HeaderBar.dividerColor.cgColor
        return layer
    }()

    let accountButton: IncreasedHitButton = {
        let button = makeHeaderBarButton(with: UIImage(named: "IconAccount"))
        button.accessibilityIdentifier = "AccountButton"
        button.accessibilityLabel = NSLocalizedString(
            "HEADER_BAR_ACCOUNT_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "HeaderBar",
            value: "Account",
            comment: ""
        )
        return button
    }()

    let settingsButton: IncreasedHitButton = {
        let button = makeHeaderBarButton(with: UIImage(named: "IconSettings"))
        button.accessibilityIdentifier = "SettingsButton"
        button.accessibilityLabel = NSLocalizedString(
            "HEADER_BAR_SETTINGS_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "HeaderBar",
            value: "Settings",
            comment: ""
        )
        return button
    }()

    class func makeHeaderBarButton(with image: UIImage?) -> IncreasedHitButton {
        let buttonImage = image?.withTintColor(UIColor.HeaderBar.buttonColor, renderingMode: .alwaysOriginal)
        let disabledButtonImage = image?.withTintColor(
            UIColor.HeaderBar.disabledButtonColor,
            renderingMode: .alwaysOriginal
        )

        let barButton = IncreasedHitButton(type: .system)
        barButton.setImage(buttonImage, for: .normal)
        barButton.setImage(disabledButtonImage, for: .disabled)
        barButton.configureForAutoLayout()

        return barButton
    }

    var showsDivider = false {
        didSet {
            if showsDivider {
                layer.addSublayer(borderLayer)
            } else {
                borderLayer.removeFromSuperlayer()
            }
        }
    }

    private var isAccountButtonHidden = false {
        didSet {
            self.accountButton.isHidden = isAccountButtonHidden
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        directionalLayoutMargins = NSDirectionalEdgeInsets(
            top: 0,
            leading: UIMetrics.contentLayoutMargins.leading,
            bottom: 0,
            trailing: UIMetrics.contentLayoutMargins.trailing
        )

        accessibilityContainerType = .semanticGroup

        let imageSize = brandNameImage?.size ?? .zero
        let brandNameAspectRatio = imageSize.width / max(imageSize.height, 1)

        [deviceName, timeLeft].forEach { deviceInfoHolder.addArrangedSubview($0) }

        addConstrainedSubviews([logoImageView, brandNameImageView, buttonContainer, deviceInfoHolder]) {
            logoImageView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor)
            logoImageView.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor)
            logoImageView.widthAnchor.constraint(equalToConstant: UIMetrics.headerBarLogoSize)
            logoImageView.heightAnchor.constraint(equalTo: logoImageView.widthAnchor, multiplier: 1)

            brandNameImageView.leadingAnchor.constraint(
                equalToSystemSpacingAfter: logoImageView.trailingAnchor,
                multiplier: 1
            )
            brandNameImageView.topAnchor.constraint(
                equalTo: layoutMarginsGuide.topAnchor,
                constant: UIMetrics.headerBarLogoSize * 0.5
            )
            brandNameImageView.widthAnchor.constraint(
                equalTo: brandNameImageView.heightAnchor,
                multiplier: brandNameAspectRatio
            )
            brandNameImageView.heightAnchor.constraint(equalToConstant: UIMetrics.headerBarBrandNameHeight)

            buttonContainer.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor)
            buttonContainer.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)

            deviceInfoHolder.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor)
            deviceInfoHolder.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
            deviceInfoHolder.topAnchor.constraint(equalToSystemSpacingBelow: logoImageView.bottomAnchor, multiplier: 1)
            layoutMarginsGuide.bottomAnchor.constraint(
                equalToSystemSpacingBelow: deviceInfoHolder.bottomAnchor,
                multiplier: 1
            )
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        borderLayer.frame = CGRect(x: 0, y: frame.maxY - 1, width: frame.width, height: 1)
        brandNameImageView.isHidden = shouldHideBrandName()
    }

    /// Returns `true` if container holding buttons intersects brand name.
    private func shouldHideBrandName() -> Bool {
        let buttonContainerRect = buttonContainer.convert(buttonContainer.bounds, to: nil)
        let brandNameRect = brandNameImageView.convert(brandNameImageView.bounds, to: nil)

        return brandNameRect.intersects(buttonContainerRect)
    }

    private func update(name: String?) {
        if let name {
            deviceName.isHidden = false
            let formattedDeviceName = NSLocalizedString(
                "DEVICE_NAME_HEADER_VIEW",
                tableName: "Account",
                value: "Device name: %@",
                comment: ""
            )
            deviceName.text = .init(format: formattedDeviceName, name)
        } else {
            deviceName.isHidden = true
        }
    }

    private func update(expiry: Date?) {
        if let expiry {
            timeLeft.isHidden = false
            let formattedTimeLeft = NSLocalizedString(
                "TIME_LEFT_HEADER_VIEW",
                tableName: "Account",
                value: "Time left: %@",
                comment: ""
            )
            timeLeft.text = .init(
                format: formattedTimeLeft,
                CustomDateComponentsFormatting.localizedString(
                    from: Date(),
                    to: expiry,
                    unitsStyle: .full
                ) ?? ""
            )
        } else {
            timeLeft.isHidden = true
        }
    }
}

extension HeaderBarView {
    func update(configuration: RootConfiguration) {
        update(name: configuration.deviceName)
        update(expiry: configuration.expiry)
        isAccountButtonHidden = !configuration.showsAccountButton
    }
}
