//
//  UITableViewCell+Disable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UITableViewCell {
    func setDisabled(_ disabled: Bool) {
        isUserInteractionEnabled = !disabled
        contentView.alpha = disabled ? 0.8 : 1
    }
}
