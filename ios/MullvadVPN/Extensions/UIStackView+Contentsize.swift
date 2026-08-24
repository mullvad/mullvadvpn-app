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
    var isOverflowed: Bool {
        return contentSize.width > bounds.width
    }

    var contentSize: CGSize {
        layoutIfNeeded()
        let availableSize = bounds.size
        guard availableSize.width > 0 else { return availableSize }
        return systemLayoutSizeFitting(
            CGSize(
                width: CGFloat.greatestFiniteMagnitude,
                height: availableSize.height),
            withHorizontalFittingPriority: .fittingSizeLevel,
            verticalFittingPriority: .fittingSizeLevel
        )
    }
}
