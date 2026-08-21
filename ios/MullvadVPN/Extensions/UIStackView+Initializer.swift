// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
