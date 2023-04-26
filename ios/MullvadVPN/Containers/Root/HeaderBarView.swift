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

    private let deviceInfoHolder: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.spacing = 16.0
        return stackView
    }()

    private lazy var deviceName: UILabel = {
        let label = UILabel(frame: .zero)
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.8)
        label.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return label
    }()

    private lazy var timeLeft: UILabel = {
        let label = UILabel(frame: .zero)
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.8)
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        return label
    }()

    let accountButton: HeaderBarButton = {
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

    let settingsButton: HeaderBarButton = {
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

    class func makeHeaderBarButton(with image: UIImage?) -> HeaderBarButton {
        let buttonImage = image?.withTintColor(UIColor.HeaderBar.buttonColor, renderingMode: .alwaysOriginal)
        let disabledButtonImage = image?.withTintColor(
            UIColor.HeaderBar.disabledButtonColor,
            renderingMode: .alwaysOriginal
        )

        let barButton = HeaderBarButton(type: .system)
        barButton.setImage(buttonImage, for: .normal)
        barButton.setImage(disabledButtonImage, for: .disabled)
        barButton.translatesAutoresizingMaskIntoConstraints = false

        return barButton
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
        directionalLayoutMargins = NSDirectionalEdgeInsets(
            top: 0,
            leading: UIMetrics.contentDirectionalLayoutMargins.leading,
            bottom: 0,
            trailing: UIMetrics.contentDirectionalLayoutMargins.trailing
        )

        accessibilityContainerType = .semanticGroup

        let imageSize = brandNameImage?.size ?? .zero
        let brandNameAspectRatio = imageSize.width / max(imageSize.height, 1)

        [deviceName, timeLeft].forEach { deviceInfoHolder.addArrangedSubview($0) }

        addConstrainedSubviews([logoImageView, brandNameImageView, accountButton, settingsButton, deviceInfoHolder]) {
            logoImageView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor)
            logoImageView.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor)
            logoImageView.widthAnchor.constraint(equalToConstant: 44)
            logoImageView.heightAnchor.constraint(
                equalTo: logoImageView.widthAnchor,
                multiplier: 1
            )

            brandNameImageView.leadingAnchor.constraint(
                equalTo: logoImageView.trailingAnchor,
                constant: 9
            )
            brandNameImageView.topAnchor.constraint(
                equalTo: layoutMarginsGuide.topAnchor,
                constant: 22
            )
            brandNameImageView.widthAnchor.constraint(
                equalTo: brandNameImageView.heightAnchor,
                multiplier: brandNameAspectRatio
            )
            brandNameImageView.heightAnchor.constraint(equalToConstant: 18)
            layoutMarginsGuide.bottomAnchor.constraint(
                equalTo: deviceInfoHolder.bottomAnchor,
                constant: 8
            )

            accountButton.leadingAnchor.constraint(
                greaterThanOrEqualTo: brandNameImageView.trailingAnchor,
                constant: 8
            )
            accountButton.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor)

            settingsButton.leadingAnchor.constraint(
                equalTo: accountButton.trailingAnchor,
                constant: 20
            ).withPriority(.defaultHigh)
            settingsButton.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
            settingsButton.centerYAnchor.constraint(equalTo: accountButton.centerYAnchor)

            deviceInfoHolder.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor)
            deviceInfoHolder.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
            deviceInfoHolder.topAnchor.constraint(equalTo: logoImageView.bottomAnchor, constant: 7)
        }

        //        addConstrainedSubviews([logoImageView, brandNameImageView, accountButton, settingsButton,
        //        deviceInfoHolder]) {
        //            logoImageView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor)
        //            logoImageView.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor)
        //            logoImageView.widthAnchor.constraint(equalToConstant: 44)
        //            logoImageView.heightAnchor.constraint(
        //                equalTo: logoImageView.widthAnchor,
        //                multiplier: 1
        //            )
        //
        //            brandNameImageView.leadingAnchor.constraint(
        //                equalTo: logoImageView.trailingAnchor,
        //                constant: 9
        //            )
        //            brandNameImageView.topAnchor.constraint(
        //                equalTo: layoutMarginsGuide.topAnchor,
        //                constant: 22
        //            )
        //            brandNameImageView.widthAnchor.constraint(
        //                equalTo: brandNameImageView.heightAnchor,
        //                multiplier: brandNameAspectRatio
        //            )
        //            brandNameImageView.heightAnchor.constraint(equalToConstant: 18)
        //            layoutMarginsGuide.bottomAnchor.constraint(
        //                equalTo: deviceInfoHolder.bottomAnchor,
        //                constant: 8
        //            )
        //
        //            accountButton.leadingAnchor.constraint(
        //                greaterThanOrEqualTo: brandNameImageView.trailingAnchor,
        //                constant: 8
        //            )
        //            accountButton.centerYAnchor.constraint(equalTo: brandNameImageView.centerYAnchor)
        //
        //            settingsButton.leadingAnchor.constraint(
        //                equalTo: accountButton.trailingAnchor,
        //                constant: 20
        //            ).withPriority(.defaultHigh)
        //            settingsButton.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
        //            settingsButton.centerYAnchor.constraint(equalTo: accountButton.centerYAnchor)
        //
        //            deviceInfoHolder.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor)
        //            deviceInfoHolder.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
        //            deviceInfoHolder.topAnchor.constraint(equalTo: logoImageView.bottomAnchor, constant: 7)
        //        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        borderLayer.frame = CGRect(x: 0, y: frame.maxY - 1, width: frame.width, height: 1)
    }
}

extension HeaderBarView {
    func update(configuration: RootConfigration) {
        if let name = configuration.deviceName {
            let formattedDeviceName = NSLocalizedString(
                "DEVICE_NAME_HEADER_VIEW",
                tableName: "Account",
                value: "Device name: %@",
                comment: ""
            )
            deviceName.text = .init(format: formattedDeviceName, name)
        }

        if let expiry = configuration.expiry {
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
        }

        deviceInfoHolder.arrangedSubviews.forEach { $0.isHidden = !configuration.showsDeviceInfo }
        accountButton.isHidden = !configuration.showsAccountButton
    }
}
