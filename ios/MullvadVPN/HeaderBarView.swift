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
    let logoImageView = UIImageView(image: UIImage(named: "LogoIcon"))

    lazy var titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.text = "MULLVAD VPN"
        titleLabel.font = UIFont.boldSystemFont(ofSize: 24)
        titleLabel.textColor = UIColor.white.withAlphaComponent(0.8)
        return titleLabel
    }()

    lazy var settingsButton: UIButton = {
        let settingsButton = UIButton(type: .custom)
        settingsButton.setImage(UIImage(named: "IconSettings"), for: .normal)
        settingsButton.accessibilityIdentifier = "SettingsButton"
        return settingsButton
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        layoutMargins = UIEdgeInsets(top: 20, left: 12, bottom: 0, right: 16)

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

        for view in [logoImageView, titleLabel, settingsButton] {
            view.translatesAutoresizingMaskIntoConstraints = false
            addSubview(view)
        }

        NSLayoutConstraint.activate(constraints)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
