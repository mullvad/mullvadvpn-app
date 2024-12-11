//
//  ChipViewModelProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI
protocol ChipViewModelProtocol: ObservableObject {
    var chips: [ChipModel] { get }
    var isExpanded: Bool { get set }
}

class FeaturesIndicatoresMockViewModel: ChipViewModelProtocol {
    @Published var chips: [ChipModel] = [
        ChipModel(name: LocalizedStringKey("DAITA")),
        ChipModel(name: LocalizedStringKey("Obfuscation")),
        ChipModel(name: LocalizedStringKey("Quantum resistance")),
        ChipModel(name: LocalizedStringKey("Multihop")),
        ChipModel(name: LocalizedStringKey("DNS content blockers")),
        ChipModel(name: LocalizedStringKey("Custom DNS")),
        ChipModel(name: LocalizedStringKey("Server IP override")),
    ]

    @Published var isExpanded: Bool

    init(isExpanded: Bool = false) {
        self.isExpanded = isExpanded
    }
}
