//
//  SelectableSettingsDetailsCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SelectableSettingsDetailsCell: SelectableSettingsCell {
    var action: (() -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: .subtitle, reuseIdentifier: reuseIdentifier)

        let actionButton = IncreasedHitButton(type: .system)
        actionButton.setImage(UIImage(systemName: "ellipsis"), for: .normal)
        actionButton.tintColor = .white

        let separatorView = UIView()
        separatorView.backgroundColor = .white

        let viewContainer = UIView()
        viewContainer.addConstrainedSubviews([separatorView, actionButton]) {
            separatorView.leadingAnchor.constraint(equalTo: viewContainer.leadingAnchor, constant: 16)
            separatorView.centerYAnchor.constraint(equalTo: viewContainer.centerYAnchor)
            separatorView.heightAnchor.constraint(equalToConstant: UIMetrics.SettingsCell.buttonSeparatorHeight)
            separatorView.widthAnchor.constraint(equalToConstant: 1)

            actionButton.pinEdgesToSuperview(.all().excluding(.leading))
            actionButton.leadingAnchor.constraint(equalTo: separatorView.trailingAnchor)
            actionButton.widthAnchor.constraint(equalToConstant: UIMetrics.SettingsCell.detailsButtonSize)
        }

        setTrailingView { superview in
            superview.addConstrainedSubviews([viewContainer]) {
                viewContainer.pinEdgesToSuperview()
            }
        }

        actionButton.addTarget(
            self,
            action: #selector(didPressActionButton),
            for: .valueChanged
        )
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Actions

    @objc private func didPressActionButton() {
        action?()
    }
}
