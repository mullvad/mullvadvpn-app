//
//  SettingsDNSInfoCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-07-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsDNSInfoCell: UITableViewCell {
    let titleLabel = UILabel()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundColor = .secondaryColor
        contentView.directionalLayoutMargins = NSDirectionalEdgeInsets(UIMetrics.SettingsRowView.footerLayoutMargins)

        titleLabel.adjustsFontForContentSizeCategory = true
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.textColor = UIColor.TableSection.footerTextColor
        titleLabel.numberOfLines = 0

        contentView.addConstrainedSubviews([titleLabel]) {
            titleLabel.pinEdgesToSuperviewMargins(.all())
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
