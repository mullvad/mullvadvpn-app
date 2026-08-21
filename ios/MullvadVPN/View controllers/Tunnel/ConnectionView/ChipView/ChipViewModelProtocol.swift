// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

protocol ChipViewModelProtocol: ObservableObject {
    var chips: [ChipModel] { get }
    func onPressed(item: ChipModel)
}

extension ChipViewModelProtocol {
    func chipsToAdd(forContainerWidth containerWidth: CGFloat) -> (chips: [ChipModel], isOverflowing: Bool) {
        var chipsToAdd = [ChipModel]()
        var isOverflowing = false

        let moreTextWidth =
            String(format: NSLocalizedString("%d more...", comment: ""), chips.count)
            .width(using: .preferredFont(forTextStyle: .subheadline)) + 4  // Some extra to be safe.
        var totalChipsWidth: CGFloat = 0

        for (index, chip) in chips.enumerated() {
            let textWidth = chip.name.width(using: .preferredFont(forTextStyle: .subheadline))
            let chipWidth =
                textWidth
                + UIMetrics.FeatureIndicators.chipViewHorizontalPadding * 2
                + UIMetrics.FeatureIndicators.chipViewTrailingMargin
            let isLastChip = index == chips.count - 1

            totalChipsWidth += chipWidth

            let chipWillFitWithMoreText = (totalChipsWidth + moreTextWidth) <= containerWidth
            let chipWillFit = totalChipsWidth <= containerWidth

            guard (chipWillFit && isLastChip) || chipWillFitWithMoreText else {
                isOverflowing = true
                break
            }

            chipsToAdd.append(chip)
        }

        return (chipsToAdd, isOverflowing)
    }
}

class MockFeatureIndicatorsViewModel: ChipViewModelProtocol {
    func onPressed(item: ChipModel) {}

    @Published var chips: [ChipModel] = [
        ChipModel(id: .daita, name: "DAITA"),
        ChipModel(id: .obfuscation, name: "Obfuscation"),
        ChipModel(id: .quantumResistance, name: "Quantum resistance"),
        ChipModel(id: .multihop, name: "Multihop"),
        ChipModel(id: .dns, name: "DNS content blockers"),
        ChipModel(id: .dns, name: "Custom DNS"),
        ChipModel(id: .ipOverrides, name: "Server IP override"),
        ChipModel(id: .includeAllNetworks, name: "Force all apps"),
    ]
}
