//
//  UIColor+Color.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

extension Color {
    static var secondaryColor = Color(UIColor.secondaryColor)

    enum Cell {
        enum Background {
            static let normal = Color(UIColor.Cell.Background.normal)
            static let disabled = Color(UIColor.Cell.Background.disabled)
            static let selected = Color(UIColor.Cell.Background.selected)
            static let disabledSelected = Color(UIColor.Cell.Background.disabledSelected)
            static let selectedAlt = Color(UIColor.Cell.Background.selectedAlt)
        }
    }
}
