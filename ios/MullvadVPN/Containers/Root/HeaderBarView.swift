//
//  HeaderBarView.swift
//  MullvadVPN
//
//  Created by pronebird on 19/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

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

    private lazy var deviceNameLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.8)
        label.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return label
    }()

    private lazy var timeLeftLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.8)
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        return label
    }()

    private lazy var buttonContainer: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [accountButton, settingsButton])
        return stackView
    }()

    private let borderLayer: CALayer = {
        let layer = CALayer()
        layer.backgroundColor = UIColor.HeaderBar.dividerColor.cgColor
        return layer
    }()

    let accountButton: UIButton = {
        let button = makeHeaderBarButton(with: UIImage(named: "IconAccount"))
        button.accessibilityIdentifier = AccessibilityIdentifier.accountButton.rawValue
        button.accessibilityLabel = NSLocalizedString(
            "HEADER_BAR_ACCOUNT_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "HeaderBar",
            value: "Account",
            comment: ""
        )
        button.heightAnchor.constraint(equalToConstant: UIMetrics.Button.barButtonSize).isActive = true
        button.widthAnchor.constraint(equalTo: button.heightAnchor, multiplier: 1).isActive = true
        return button
    }()

    let settingsButton: UIButton = {
        let button = makeHeaderBarButton(with: UIImage(named: "IconSettings"))
        button.accessibilityIdentifier = AccessibilityIdentifier.settingsButton.rawValue
        button.accessibilityLabel = NSLocalizedString(
            "HEADER_BAR_SETTINGS_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "HeaderBar",
            value: "Settings",
            comment: ""
        )
        button.heightAnchor.constraint(equalToConstant: UIMetrics.Button.barButtonSize).isActive = true
        button.widthAnchor.constraint(equalTo: button.heightAnchor, multiplier: 1).isActive = true
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

    var isDeviceInfoHidden = false {
        didSet {
            deviceInfoHolder.arrangedSubviews.forEach { $0.isHidden = isDeviceInfoHidden }
        }
    }

    private var isAccountButtonHidden = false {
        didSet {
            accountButton.isHidden = isAccountButtonHidden
        }
    }

    private var timeLeft: Date? {
        didSet {
            if let timeLeft {
                let formattedTimeLeft = NSLocalizedString(
                    "TIME_LEFT_HEADER_VIEW",
                    tableName: "Account",
                    value: "Time left: %@",
                    comment: ""
                )
                timeLeftLabel.text = String(
                    format: formattedTimeLeft,
                    CustomDateComponentsFormatting.localizedString(
                        from: Date(),
                        to: timeLeft,
                        unitsStyle: .full
                    ) ?? ""
                )
            } else {
                timeLeftLabel.text = ""
            }
        }
    }

    private var deviceName: String? {
        didSet {
            if let deviceName {
                let formattedDeviceName = NSLocalizedString(
                    "DEVICE_NAME_HEADER_VIEW",
                    tableName: "Account",
                    value: "Device name: %@",
                    comment: ""
                )
                deviceNameLabel.text = String(format: formattedDeviceName, deviceName)
            } else {
                deviceNameLabel.text = ""
            }
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

        let brandImageSize = brandNameImage?.size ?? .zero
        let brandNameAspectRatio = brandImageSize.width / max(brandImageSize.height, 1)

        var buttonContainerTrailingAdjustment: CGFloat = 0
        if let buttonImageWidth = settingsButton.currentImage?.size.width {
            buttonContainerTrailingAdjustment = max((UIMetrics.Button.barButtonSize - buttonImageWidth) / 2, 0)
        }

        [deviceNameLabel, timeLeftLabel].forEach { deviceInfoHolder.addArrangedSubview($0) }

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
            buttonContainer.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor,
                constant: buttonContainerTrailingAdjustment
            )

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

    func update(configuration: RootConfiguration) {
        deviceName = configuration.deviceName
        timeLeft = configuration.expiry
        isAccountButtonHidden = !configuration.showsAccountButton
    }
}
