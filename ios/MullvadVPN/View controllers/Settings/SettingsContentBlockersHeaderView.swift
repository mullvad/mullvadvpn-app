//
//  SettingsContentBlockersHeaderView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-06.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsContentBlockersHeaderView: UITableViewHeaderFooterView {
    let titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 18)
        titleLabel.textColor = .white
        titleLabel.numberOfLines = 0
        return titleLabel
    }()

    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        contentView.layoutMargins = UIMetrics.settingsCellLayoutMargins
        contentView.addSubview(titleLabel)

        contentView.addConstraints([
            titleLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            titleLabel.leadingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),
            titleLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor)
                .withPriority(.defaultLow),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
