//
//  AddLocationCellViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-03-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct AddLocationCellViewModel: Hashable {
    let node: LocationNode
    var indentationLevel = 0
    var isSelected: Bool
}
