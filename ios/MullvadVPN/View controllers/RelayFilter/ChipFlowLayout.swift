//
//  ChipFlowLayout.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ChipFlowLayout: UICollectionViewCompositionalLayout {
    init() {
        super.init { _, _ -> NSCollectionLayoutSection? in
            // Create an item with flexible size
            let itemSize = NSCollectionLayoutSize(widthDimension: .estimated(50), heightDimension: .estimated(20))
            let item = NSCollectionLayoutItem(layoutSize: itemSize)
            item.edgeSpacing = NSCollectionLayoutEdgeSpacing(
                leading: .fixed(0),
                top: .fixed(0),
                trailing: .fixed(0),
                bottom: .fixed(0)
            )

            // Create a group that fills the available width and wraps items with proper spacing
            let groupSize = NSCollectionLayoutSize(
                widthDimension: .fractionalWidth(1.0),
                heightDimension: .estimated(20)
            )
            let group = NSCollectionLayoutGroup.horizontal(layoutSize: groupSize, subitems: [item])
            group.interItemSpacing = .fixed(UIMetrics.FilterView.interChipViewSpacing)
            group.contentInsets = .zero

            // Create a section with zero inter-group spacing and no content insets
            let section = NSCollectionLayoutSection(group: group)
            section.interGroupSpacing = UIMetrics.FilterView.interChipViewSpacing
            section.contentInsets = .zero

            return section
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
