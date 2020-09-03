//
//  BasicTableViewCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class BasicTableViewCell: UITableViewCell {

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        let backgroundView = UIView()
        backgroundView.backgroundColor = UIColor.Cell.backgroundColor

        let selectedBackgroundView = UIView()
        selectedBackgroundView.backgroundColor = UIColor.Cell.selectedBackgroundColor

        self.backgroundView = backgroundView
        self.selectedBackgroundView = selectedBackgroundView
        backgroundColor = UIColor.clear
        contentView.backgroundColor = UIColor.clear
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

}
