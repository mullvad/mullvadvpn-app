//
//  NSLayoutConstraint+Helpers.swift
//  MullvadVPN
//
//  Created by pronebird on 21/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSLayoutConstraint {
    /// Sets constraint priority and returns `self`
    func withPriority(_ priority: UILayoutPriority) -> Self {
        self.priority = priority
        return self
    }

    /// Stores constraint into array and returns `self`.
    func store(into outConstraints: inout [NSLayoutConstraint]) -> Self {
        outConstraints.append(self)
        return self
    }
}
