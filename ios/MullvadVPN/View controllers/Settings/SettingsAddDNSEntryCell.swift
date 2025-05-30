//
//  SettingsAddDNSEntryCell.swift
//  MullvadVPN
//
//  Created by pronebird on 27/10/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsAddDNSEntryCell: SettingsCell {
    var tapAction: (() -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView?.backgroundColor = UIColor.Cell.Background.indentationLevelZero

        let gestureRecognizer = UITapGestureRecognizer(
            target: self,
            action: #selector(handleTap(_:))
        )
        contentView.addGestureRecognizer(gestureRecognizer)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc func handleTap(_ sender: UIGestureRecognizer) {
        if case .ended = sender.state {
            tapAction?()
        }
    }
}
