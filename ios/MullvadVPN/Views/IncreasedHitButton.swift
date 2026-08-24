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
import UIKit

final class IncreasedHitButton: UIButton {
    private let defaultSize = UIMetrics.Button.minimumTappableAreaSize.width

    override func point(inside point: CGPoint, with event: UIEvent?) -> Bool {
        let width = bounds.width
        let height = bounds.height
        let dx = (max(defaultSize, width) - width) * 0.5
        let dy = (max(defaultSize, height) - height) * 0.5
        return bounds.insetBy(dx: -dx, dy: -dy).contains(point)
    }
}
