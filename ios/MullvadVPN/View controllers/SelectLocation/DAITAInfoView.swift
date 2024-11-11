//
//  DAITAInfoView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

class DAITAInfoView: UIView {
    let infoLabel: UILabel = {
        let label = UILabel()
        label.numberOfLines = 0

        let infoTextParagraphStyle = NSMutableParagraphStyle()
        infoTextParagraphStyle.lineSpacing = 1.3
        infoTextParagraphStyle.alignment = .center

        label.attributedText = NSAttributedString(
            string: NSLocalizedString(
                "SELECT_LOCATION_DAITA_INFO",
                tableName: "SelectLocation",
                value: """
                The entry server for multihop is currently overridden by DAITA. \
                To select an entry server, please first enable “Direct only” or disable “DAITA” in the settings.
                """,
                comment: ""
            ),
            attributes: [
                .font: UIFont.systemFont(ofSize: 15),
                .foregroundColor: UIColor.white,
                .paragraphStyle: infoTextParagraphStyle,
            ]
        )

        return label
    }()

    let settingsButton: UIButton = {
        let settingsButton = AppButton(style: .default)
        settingsButton.setTitle(NSLocalizedString(
            "SELECT_LOCATION_DAITA_BUTTON",
            tableName: "SelectLocation",
            value: "Open DAITA settings",
            comment: ""
        ), for: .normal)

        return settingsButton
    }()

    var didPressDaitaSettingsButton: (() -> Void)?

    init() {
        super.init(frame: .zero)

        backgroundColor = .secondaryColor
        layoutMargins = UIMetrics.contentInsets

        settingsButton.addTarget(self, action: #selector(didPressButton), for: .touchUpInside)

        addConstrainedSubviews([infoLabel, settingsButton]) {
            infoLabel.pinEdgesToSuperviewMargins(.init([.leading(24), .trailing(24)]))

            settingsButton.pinEdgesToSuperviewMargins(.init([.leading(0), .trailing(0)]))
            settingsButton.topAnchor.constraint(equalTo: infoLabel.bottomAnchor, constant: 32)
            settingsButton.bottomAnchor.constraint(equalTo: centerYAnchor)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func didPressButton() {
        didPressDaitaSettingsButton?()
    }
}
