//
//  LocationSectionHeaderView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class LocationSectionHeaderFooterView: UITableViewHeaderFooterView {
    static let reuseIdentifier = "LocationSectionHeaderFooterView"

    private let label = UILabel()
    private let button = UIButton(type: .system)

    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        contentView.backgroundColor = .primaryColor

        directionalLayoutMargins = NSDirectionalEdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 8)

        // Configure button
        button.setImage(UIImage(systemName: "ellipsis"), for: .normal)
        button.tintColor = UIColor(white: 1, alpha: 0.6)

        // Add subviews
        contentView.addSubview(label)
        contentView.addSubview(button)

        label.translatesAutoresizingMaskIntoConstraints = false
        button.translatesAutoresizingMaskIntoConstraints = false

        // Setup constraints
        NSLayoutConstraint.activate([
            // Label constraints: pinned to top, bottom, and leading margins of contentView
            label.leadingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            label.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            label.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),

            // Button constraints: trailing margin and vertical center
            button.leadingAnchor.constraint(greaterThanOrEqualTo: label.trailingAnchor, constant: 8),
            button.trailingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),
            button.centerYAnchor.constraint(equalTo: contentView.centerYAnchor),
            button.widthAnchor.constraint(equalTo: button.heightAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configure(text: String, buttonAction: UIAction?) {
        var contentConfig = UIListContentConfiguration.groupedHeader()
        contentConfig.text = text
        contentConfig.textProperties.color = .primaryTextColor
        contentConfig.textProperties.font = .mullvadSmall
        contentConfig.textProperties.adjustsFontForContentSizeCategory = true

        // Apply the font and color directly to the label:
        label.text = text
        label.font = contentConfig.textProperties.font
        label.textColor = contentConfig.textProperties.color
        label.adjustsFontForContentSizeCategory = contentConfig.textProperties.adjustsFontForContentSizeCategory
        label.numberOfLines = 0
        label.lineBreakMode = .byWordWrapping
        label.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)

        if let action = buttonAction {
            button.isHidden = false
            button.removeTarget(nil, action: nil, for: .allEvents)
            button.addAction(action, for: .touchUpInside)
        } else {
            button.isHidden = true
        }
    }
}
