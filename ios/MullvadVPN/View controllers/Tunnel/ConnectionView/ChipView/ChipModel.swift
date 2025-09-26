//
//  FeatureChipModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

struct ChipModel: Identifiable {
    var id: FeatureType
    let name: String
    let isMultihopEverywhere: Bool
}
