//
//  SelectableSettingsCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SelectableSettingsCell: SettingsCell {
    let tickImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "IconTick"))
        imageView.tintColor = .white
        imageView.alpha = 0
        return imageView
    }()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        setLeftView(tickImageView, spacing: UIMetrics.SettingsCell.selectableSettingsCellLeftViewSpacing)
        selectedBackgroundView?.backgroundColor = UIColor.Cell.Background.selected
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setLeftView(tickImageView, spacing: UIMetrics.SettingsCell.selectableSettingsCellLeftViewSpacing)
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        tickImageView.alpha = selected ? 1 : 0
    }
}
