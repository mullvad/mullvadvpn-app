//
//  UIStackView+Initializer.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-05-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIStackView {
    /// a convenience initialiser allowing the elimination of constructor closures
    convenience init(
        axis: NSLayoutConstraint.Axis = .horizontal,
        alignment: UIStackView.Alignment = .fill,
        distribution: UIStackView.Distribution = .fill,
        isLayoutMarginsRelativeArrangement: Bool = false,
        spacing: CGFloat = 0.0,
    ) {
        self.init()
        self.axis = axis
        self.alignment = alignment
        self.distribution = distribution
        self.isLayoutMarginsRelativeArrangement = isLayoutMarginsRelativeArrangement
        self.spacing = spacing
    }
}
