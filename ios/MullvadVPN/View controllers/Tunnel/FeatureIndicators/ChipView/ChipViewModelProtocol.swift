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
}

extension ChipViewModelProtocol {
    func chipsToAdd(forContainerWidth containerWidth: CGFloat) -> (chips: [ChipModel], isOverflowing: Bool) {
        var chipsToAdd = [ChipModel]()
        var isOverflowing = false

        let moreTextWidth = String(
            format: NSLocalizedString(
                "CONNECTION_VIEW_CHIPS_MORE",
                tableName: "ConnectionView",
                value: "@d more...",
                comment: ""
            ), arguments: [chips.count]
        )
        .width(using: .preferredFont(forTextStyle: .subheadline)) + 4 // Some extra to be safe.
        var totalChipsWidth: CGFloat = 0

        for (index, chip) in chips.enumerated() {
            let textWidth = chip.name.width(using: .preferredFont(forTextStyle: .subheadline))
            let chipWidth = textWidth
                + UIMetrics.FeatureIndicators.chipViewHorisontalPadding * 2
                + UIMetrics.FeatureIndicators.chipViewTrailingMargin
            let isLastChip = index == chips.count - 1

            totalChipsWidth += chipWidth

            let chipWillFitWithMoreText = (totalChipsWidth + moreTextWidth) <= containerWidth
            let chipWillFit = totalChipsWidth <= containerWidth

            guard (chipWillFit && isLastChip) || chipWillFitWithMoreText else {
                isOverflowing = true
                break
            }
        }

        return (chipsToAdd, isOverflowing)
    }
}

class MockFeatureIndicatorsViewModel: ChipViewModelProtocol {
    @Published var chips: [ChipModel] = [
        ChipModel(name: "DAITA"),
        ChipModel(name: "Obfuscation"),
        ChipModel(name: "Quantum resistance"),
        ChipModel(name: "Multihop"),
        ChipModel(name: "DNS content blockers"),
        ChipModel(name: "Custom DNS"),
        ChipModel(name: "Server IP override"),
    ]
}
