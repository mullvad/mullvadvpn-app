//
//  ChangeLogNotifierTableCellView.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-14.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class ChangeLogNotifierTableCellView: UITableViewCell {
    private lazy var dotLabel: UIView = {
        let label = UILabel()
        label.text = "\u{2022}"
        label.font = UIFont.systemFont(ofSize: 16)
        label.textColor = .white
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private lazy var changeDescriptionLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 14)
        label.textColor = .white
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        addAndConstraintViews()

        backgroundColor = .clear
        selectionStyle = .none

        // Not allowing accessibility for the cell itself
        isAccessibilityElement = false
        // Allowing accessibility in the cell's subviews
        accessibilityElements = [changeDescriptionLabel]
    }

    private func addAndConstraintViews() {
        [dotLabel, changeDescriptionLabel].forEach(contentView.addSubview)

        NSLayoutConstraint.activate([
            dotLabel.topAnchor.constraint(equalTo: contentView.topAnchor),
            dotLabel.leadingAnchor.constraint(
                equalTo: contentView.leadingAnchor,
                constant: 8
            ),
            dotLabel.widthAnchor.constraint(equalToConstant: 10),
            dotLabel.heightAnchor.constraint(equalToConstant: 14),

            changeDescriptionLabel.topAnchor.constraint(equalTo: contentView.topAnchor),
            changeDescriptionLabel.leadingAnchor.constraint(
                equalTo: dotLabel.trailingAnchor,
                constant: 8
            ),
            changeDescriptionLabel.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            changeDescriptionLabel.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),
        ])
    }

    public func setChange(_ change: String) {
        changeDescriptionLabel.text = change

        changeDescriptionLabel.accessibilityIdentifier =
            "ChangeLogNotifierTableCellView.changeDescriptionLabel" + change
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
