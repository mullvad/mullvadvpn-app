//
//  IncreasedTapAreaButton.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-05-16.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class SquareHitButton: UIButton {
    private let defaultSize = 44.0

    override func point(inside point: CGPoint, with event: UIEvent?) -> Bool {
        let width = bounds.width
        let maxWidth = max(defaultSize, width)
        let margin = (maxWidth - width) * 0.5
        return bounds.insetBy(dx: -margin, dy: -margin).contains(point)
    }
}
