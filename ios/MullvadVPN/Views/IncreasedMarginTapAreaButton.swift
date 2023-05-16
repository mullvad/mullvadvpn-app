//
//  IncreasedTapAreaButton.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-05-16.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class IncreasedMarginTapAreaButton: UIButton {
    var margin: CGFloat = 0.0

    init(margin: CGFloat) {
        super.init(frame: .zero)
        self.margin = margin
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        margin = 0.0
    }

    override func point(inside point: CGPoint, with event: UIEvent?) -> Bool {
        bounds.insetBy(dx: -margin, dy: -margin).contains(point)
    }
}
