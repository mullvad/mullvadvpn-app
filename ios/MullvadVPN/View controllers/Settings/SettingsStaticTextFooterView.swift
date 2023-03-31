//
//  SettingsStaticTextFooterView.swift
//  MullvadVPN
//
//  Created by pronebird on 05/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsStaticTextFooterView: UITableViewHeaderFooterView {
    let titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.font = UIFont.systemFont(ofSize: 14)
        titleLabel.textColor = .white
        titleLabel.numberOfLines = 0
        return titleLabel
    }()

    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        contentView.layoutMargins = UIMetrics.settingsCellLayoutMargins

        contentView.addConstrainedSubviews([titleLabel]) {
            titleLabel.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            titleLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor)
                .withPriority(.defaultLow)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
