//
//  Color.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension Color {
    /// Returns the color darker by the given percent (in range from 0..1)
    func darkened(by percent: CGFloat) -> Color? {
        UIColor(self).darkened(by: percent)?.color
    }
}
