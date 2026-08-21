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

class CheckboxView: UIView {
    private let checkboxSelectedView: UIImageView = {
        UIImageView(image: UIImage.checkboxSelected)
    }()

    private let checkboxUnselectedView: UIImageView = {
        UIImageView(image: UIImage.checkboxUnselected)
    }()

    var isChecked = false {
        didSet {
            checkboxSelectedView.alpha = isChecked ? 1 : 0
        }
    }

    init() {
        super.init(frame: .zero)

        addConstrainedSubviews([checkboxSelectedView, checkboxUnselectedView]) {
            checkboxSelectedView.pinEdgesToSuperview()
            checkboxUnselectedView.pinEdgesToSuperview()
        }
        checkboxSelectedView.adjustsImageSizeForAccessibilityContentSizeCategory = true
        checkboxUnselectedView.adjustsImageSizeForAccessibilityContentSizeCategory = true
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
