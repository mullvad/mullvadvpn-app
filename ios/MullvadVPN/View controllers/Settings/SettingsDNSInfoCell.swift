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
        contentView.directionalLayoutMargins = UIMetrics.SettingsCell.defaultLayoutMargins

        titleLabel.adjustsFontForContentSizeCategory = true
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.textColor = UIColor.TableSection.footerTextColor
        titleLabel.numberOfLines = 0
        titleLabel.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)
        titleLabel.setContentHuggingPriority(.defaultLow, for: .vertical)

        contentView.addConstrainedSubviews([titleLabel]) {
            titleLabel.pinEdgesToSuperviewMargins(.all().excluding([.leading]))
            titleLabel.pinEdgesToSuperview(.init([.leading(16)]))
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
