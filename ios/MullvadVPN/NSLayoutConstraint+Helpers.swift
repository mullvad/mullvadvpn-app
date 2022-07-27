//
//  NSLayoutConstraint+Helpers.swift
//  MullvadVPN
//
//  Created by pronebird on 21/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSLayoutConstraint {
    func withPriority(_ priority: UILayoutPriority) -> Self {
        self.priority = priority
        return self
    }
}
