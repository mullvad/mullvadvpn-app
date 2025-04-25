//
//  CheckableSettingsCell.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CheckableSettingsCell: SettingsCell {
    let checkboxView = CheckboxView()

    var isEnabled = true {
        didSet {
            titleLabel.isEnabled = isEnabled
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        setCheckboxView()
        selectedBackgroundView?.backgroundColor = .clear
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()
        setCheckboxView()
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        checkboxView.isChecked = selected
    }

    override func applySubCellStyling() {
        super.applySubCellStyling()

        contentView.layoutMargins.left = 0
    }

    private func setCheckboxView() {
        setLeadingView { superview in
            superview.addConstrainedSubviews([checkboxView]) {
                checkboxView.centerYAnchor.constraint(equalTo: superview.centerYAnchor)
                checkboxView.pinEdgesToSuperview(PinnableEdges([
                    .leading(0),
                    .trailing(UIMetrics.SettingsCell.checkableSettingsCellLeftViewSpacing),
                ]))
            }
        }
    }
}
