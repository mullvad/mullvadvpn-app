//
//  ChipFlowLayout.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ChipFlowLayout: UICollectionViewFlowLayout {
    override init() {
        super.init()
        estimatedItemSize = UICollectionViewFlowLayout.automaticSize
        scrollDirection = .vertical
        minimumInteritemSpacing = UIMetrics.FilterView.interChipViewSpacing
        minimumLineSpacing = UIMetrics.FilterView.interChipViewSpacing
        sectionInset = .zero
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutAttributesForElements(in rect: CGRect) -> [UICollectionViewLayoutAttributes]? {
        guard let originalAttributes = super.layoutAttributesForElements(in: rect) else {
            return nil
        }

        let attributes = originalAttributes.compactMap { $0.copy() as? UICollectionViewLayoutAttributes }

        // Detect RTL
        let languageCode = Locale.current.language.languageCode?.identifier ?? "en"
        let isRTL = Locale.Language(identifier: languageCode).characterDirection == .rightToLeft

        var currentLineY: CGFloat = -1
        var currentLineAttributes: [UICollectionViewLayoutAttributes] = []

        for attribute in attributes where attribute.representedElementCategory == .cell {
            if abs(attribute.frame.origin.y - currentLineY) > 1 {
                // Align previous line before starting new
                align(attributes: currentLineAttributes, isRTL: isRTL)
                currentLineY = attribute.frame.origin.y
                currentLineAttributes = [attribute]
            } else {
                currentLineAttributes.append(attribute)
            }
        }

        // Align last line
        align(attributes: currentLineAttributes, isRTL: isRTL)

        return attributes
    }

    private func align(attributes: [UICollectionViewLayoutAttributes], isRTL: Bool) {
        guard !attributes.isEmpty else { return }

        var currentX = isRTL ? collectionViewContentSize.width - sectionInset.right : sectionInset.left

        for attr in isRTL ? attributes.reversed() : attributes {
            var frame = attr.frame
            frame.origin.x = currentX - (isRTL ? frame.width : 0)
            attr.frame = frame
            currentX += (isRTL ? -1 : 1) * (frame.width + minimumInteritemSpacing)
        }
    }
}
