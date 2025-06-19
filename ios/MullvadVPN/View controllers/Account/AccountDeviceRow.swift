//
//  AccountDeviceRow.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AccountDeviceRow: UIView {
    var deviceName: String? {
        didSet {
            deviceLabel.text = deviceName?.capitalized ?? ""
            accessibilityValue = deviceName
            setAccessibilityIdentifier(.accountPageDeviceNameLabel)
        }
    }

    var deviceManagementButtonAction: (() -> Void)?

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "DEVICE_NAME",
            tableName: "Account",
            value: "Device name",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = UIColor(white: 1.0, alpha: 0.6)
        return label
    }()

    private let deviceLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        return label
    }()

    private let deviceManagementButton: UIButton = {
        let button = IncreasedHitButton(type: .system)
        button.isExclusiveTouch = true
        button.setAccessibilityIdentifier(.deviceManagementButton)
        let attributes: [NSAttributedString.Key: Any] = [
            .font: UIFont.mullvadSmallSemiBold,
            .foregroundColor: UIColor.primaryTextColor,
            .underlineStyle: NSUnderlineStyle.single.rawValue,
        ]
        let title = NSLocalizedString(
            "DEVICE_MANAGEMENT",
            tableName: "Account",
            value: "Manage devices",
            comment: ""
        )
        let attributeString = NSMutableAttributedString(
            string: title,
            attributes: attributes
        )
        button.setAttributedTitle(attributeString, for: .normal)
        return button
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        let contentContainerView = UIStackView(arrangedSubviews: [titleLabel, deviceLabel])
        contentContainerView.axis = .vertical
        contentContainerView.alignment = .leading
        contentContainerView.spacing = 8

        addConstrainedSubviews(
            [contentContainerView, deviceManagementButton]
        ) {
            contentContainerView.pinEdgesToSuperview()
            deviceManagementButton.centerYAnchor.constraint(equalTo: deviceLabel.centerYAnchor)
            deviceManagementButton.pinEdgeToSuperview(.trailing(0))
        }

        isAccessibilityElement = true
        accessibilityLabel = titleLabel.text

        deviceManagementButton.addTarget(
            self,
            action: #selector(didTapDeviceManagementButton),
            for: .touchUpInside
        )
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setButtons(enabled: Bool) {
        deviceManagementButton.isEnabled = enabled
    }

    @objc private func didTapDeviceManagementButton() {
        deviceManagementButtonAction?()
    }
}
