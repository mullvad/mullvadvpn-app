//
//  EmptyTableViewHeaderFooterView.swift
//  MullvadVPN
//
//  Created by pronebird on 27/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class EmptyTableViewHeaderFooterView: UITableViewHeaderFooterView {
    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        textLabel?.isHidden = true
        contentView.backgroundColor = .clear
        backgroundView?.backgroundColor = .clear
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
