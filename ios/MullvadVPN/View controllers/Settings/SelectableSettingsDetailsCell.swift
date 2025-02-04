//
//  SelectableSettingsDetailsCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SelectableSettingsDetailsCell: SelectableSettingsCell {
    let viewContainer = UIView()

    var buttonAction: (() -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: .subtitle, reuseIdentifier: reuseIdentifier)

        let actionButton = IncreasedHitButton(type: .system)
        var actionButtonConfiguration = actionButton.configuration ?? .plain()
        actionButtonConfiguration.image = UIImage(systemName: "ellipsis")?
            .withRenderingMode(.alwaysOriginal)
            .withTintColor(.white)
        actionButton.configuration = actionButtonConfiguration
        actionButton.setAccessibilityIdentifier(.openPortSelectorMenuButton)

        actionButton.addTarget(
            self,
            action: #selector(didPressActionButton),
            for: .touchUpInside
        )

        let separatorView = UIView()
        separatorView.backgroundColor = .secondaryColor

        viewContainer.addConstrainedSubviews([separatorView, actionButton]) {
            separatorView.leadingAnchor.constraint(equalTo: viewContainer.leadingAnchor, constant: 16)
            separatorView.centerYAnchor.constraint(equalTo: viewContainer.centerYAnchor)
            separatorView.heightAnchor.constraint(equalToConstant: UIMetrics.SettingsCell.verticalDividerHeight)
            separatorView.widthAnchor.constraint(equalToConstant: 1)

            actionButton.pinEdgesToSuperview(.all().excluding(.leading))
            actionButton.leadingAnchor.constraint(equalTo: separatorView.trailingAnchor)
            actionButton.widthAnchor.constraint(equalToConstant: UIMetrics.SettingsCell.detailsButtonSize)
        }

        setViewContainer()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()
        setViewContainer()
    }

    private func setViewContainer() {
        setTrailingView { superview in
            superview.addConstrainedSubviews([viewContainer]) {
                viewContainer.pinEdgesToSuperview()
            }
        }
    }

    // MARK: - Actions

    @objc private func didPressActionButton() {
        buttonAction?()
    }
}
