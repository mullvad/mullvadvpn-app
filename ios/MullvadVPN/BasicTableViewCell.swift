//
//  BasicTableViewCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class BasicTableViewCell: UITableViewCell {

    override func awakeFromNib() {
        super.awakeFromNib()

        let backgroundView = UIView()
        backgroundView.backgroundColor = UIColor.cellBackgroundColor

        let selectedBackgroundView = UIView()
        selectedBackgroundView.backgroundColor = UIColor.cellSelectedBackgroundColor

        self.backgroundView = backgroundView
        self.selectedBackgroundView = selectedBackgroundView
        backgroundColor = UIColor.clear
        contentView.backgroundColor = UIColor.clear
    }
}
