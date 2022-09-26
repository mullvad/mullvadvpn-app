//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum SettingsDisclosureType {
    case none
    case chevron
    case externalLink
    case tick

    var image: UIImage? {
        switch self {
        case .none:
            return nil
        case .chevron:
            return UIImage(named: "IconChevron")
        case .externalLink:
            return UIImage(named: "IconExtlink")
        case .tick:
            return UIImage(named: "IconTickSml")
        }
    }
}

class SettingsCell: UITableViewCell {
    let titleLabel = UILabel()
    let detailTitleLabel = UILabel()
    let disclosureImageView = UIImageView(image: nil)

    var disclosureType: SettingsDisclosureType = .none {
        didSet {
            accessoryType = .none

            let image = disclosureType.image?.withTintColor(
                UIColor.Cell.disclosureIndicatorColor,
                renderingMode: .alwaysOriginal
            )

            if let image = image {
                disclosureImageView.image = image
                disclosureImageView.sizeToFit()
                accessoryView = disclosureImageView
            } else {
                accessoryView = nil
            }
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView = UIView()
        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor

        selectedBackgroundView = UIView()
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor

        separatorInset = .zero
        backgroundColor = .clear
        contentView.backgroundColor = .clear

        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 17)
        titleLabel.textColor = UIColor.Cell.titleTextColor

        detailTitleLabel.translatesAutoresizingMaskIntoConstraints = false
        detailTitleLabel.font = UIFont.systemFont(ofSize: 13)
        detailTitleLabel.textColor = UIColor.Cell.detailTextColor

        titleLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentHuggingPriority(.defaultLow, for: .horizontal)

        titleLabel.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        contentView.addSubview(titleLabel)
        contentView.addSubview(detailTitleLabel)

        setLayoutMargins()

        NSLayoutConstraint.activate([
            titleLabel.leadingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            titleLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            titleLabel.bottomAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),

            detailTitleLabel.leadingAnchor.constraint(
                greaterThanOrEqualToSystemSpacingAfter: titleLabel.trailingAnchor,
                multiplier: 1
            ),

            detailTitleLabel.trailingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),
            detailTitleLabel.topAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            detailTitleLabel.bottomAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setLayoutMargins()
    }

    private func setLayoutMargins() {
        // Set layout margins for standard acceessories added into the cell (reorder control, etc..)
        layoutMargins = UIMetrics.settingsCellLayoutMargins

        // Set layout margins for cell content
        contentView.layoutMargins = UIMetrics.settingsCellLayoutMargins
    }
}
