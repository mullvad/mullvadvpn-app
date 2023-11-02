//
//  NSDirectionalEdgeInsets+Helpers.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSDirectionalEdgeInsets {
    /// Converts directional edge insets to `UIEdgeInsets` based on interface direction.
    func toEdgeInsets(_ interfaceDirection: UIUserInterfaceLayoutDirection) -> UIEdgeInsets {
        UIEdgeInsets(
            top: top,
            left: interfaceDirection == .rightToLeft ? trailing : leading,
            bottom: bottom,
            right: interfaceDirection == .rightToLeft ? leading : trailing
        )
    }
}
