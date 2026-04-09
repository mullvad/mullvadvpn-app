//
//  FeatureChipModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

enum ChipStyle {
    case standard
    #if NEVER_IN_PRODUCTION
    case rainbowShimmer
    #endif
}

struct ChipModel: Identifiable {
    var id: FeatureType
    let name: String
    var icon: Image? = nil
    var style: ChipStyle = .standard
}
