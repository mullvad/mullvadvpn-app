//
//  CGSize+Helpers.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-10-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension CGSize {
    // Function to deduct insets from CGSize
    func deducting(insets: NSDirectionalEdgeInsets) -> CGSize {
        let newWidth = width - (insets.leading + insets.trailing)
        let newHeight = height - (insets.top + insets.bottom)
        return CGSize(width: newWidth, height: newHeight)
    }
}
