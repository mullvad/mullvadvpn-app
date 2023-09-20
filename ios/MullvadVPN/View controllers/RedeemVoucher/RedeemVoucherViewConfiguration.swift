//
//  RedeemVoucherViewConfiguration.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-09-12.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

struct RedeemVoucherViewConfiguration {
    let adjustViewWhenKeyboardAppears: Bool
    /// Hides the title when set to `true`.
    let shouldUseCompactStyle: Bool
    /// Custom  margins to use for the compact style.
    let layoutMargins: NSDirectionalEdgeInsets
}
