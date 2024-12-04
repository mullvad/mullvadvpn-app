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
    func chipsToAdd(forContainerWidth containerWidth: CGFloat) -> (chips: [ChipModel], chipsWillOverflow: Bool) {
        var chipsToAdd = [ChipModel]()
        var chipsWillOverflow = false

        let moreTextWidth = "\(chips.count) more..."
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

            if chipWillFitWithMoreText {
                // If a chip can fit together with the "more" text, add it.
                chipsToAdd.append(chip)
                chipsWillOverflow = !isLastChip
            } else if chipWillFit && isLastChip {
                // If a chip can fit and it's the last one, add it.
                chipsToAdd.append(chip)
                chipsWillOverflow = false
            } else {
                break
            }
        }

        return (chipsToAdd, chipsWillOverflow)
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
