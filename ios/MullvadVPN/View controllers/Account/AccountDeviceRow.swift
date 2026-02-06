//
//  AccountDeviceRow.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
        label.text = NSLocalizedString("Device name", comment: "")
        label.font = .mullvadTiny
        label.adjustsFontForContentSizeCategory = true
        label.textColor = UIColor(white: 1.0, alpha: 0.6)
        return label
    }()

    private let deviceLabel: UILabel = {
        let label = UILabel()
        label.font = .mullvadSmall
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    private let deviceManagementButton: UILabel = {
        let button = UILabel()
        button.adjustsFontForContentSizeCategory = true
        button.isUserInteractionEnabled = true
        button.numberOfLines = 0
        button.textAlignment = .center

        let attributes: [NSAttributedString.Key: Any] = [
            .font: UIFont.mullvadSmallSemiBold,
            .foregroundColor: UIColor.primaryTextColor,
            .underlineStyle: NSUnderlineStyle.single.rawValue,
        ]
        let title = NSLocalizedString("Manage devices", comment: "")
        button.attributedText = NSMutableAttributedString(
            string: title,
            attributes: attributes
        )
        button.setAccessibilityIdentifier(.deviceManagementButton)
        return button
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        let contentContainerView = UIStackView(arrangedSubviews: [titleLabel, deviceLabel])
        contentContainerView.axis = .vertical
        contentContainerView.alignment = .leading
        contentContainerView.spacing = 8

        contentContainerView.setContentCompressionResistancePriority(.required, for: .horizontal)
        contentContainerView.setContentHuggingPriority(.defaultHigh, for: .horizontal)

        deviceManagementButton.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        deviceManagementButton.setContentHuggingPriority(.defaultHigh, for: .horizontal)

        addConstrainedSubviews(
            [contentContainerView, deviceManagementButton]
        ) {
            contentContainerView.pinEdgesToSuperview(PinnableEdges([.leading(0), .bottom(0), .top(0)]))
            deviceManagementButton.topAnchor.constraint(equalTo: deviceLabel.topAnchor)
            deviceManagementButton.pinEdgesToSuperview(PinnableEdges([.trailing(0), .bottom(0)]))
            deviceManagementButton.leadingAnchor.constraint(equalTo: contentContainerView.trailingAnchor, constant: 16)
        }

        isAccessibilityElement = true
        accessibilityLabel = titleLabel.text

        deviceManagementButton.addGestureRecognizer(
            UITapGestureRecognizer(
                target: self,
                action: #selector(didTapDeviceManagementButton)
            ))
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
