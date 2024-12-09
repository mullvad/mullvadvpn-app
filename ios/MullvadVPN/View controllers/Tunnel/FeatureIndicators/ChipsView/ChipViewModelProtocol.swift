//
//  ChipViewModelProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
protocol ChipViewModelProtocol: ObservableObject {
    var chips: [ChipModel] { get }
}
