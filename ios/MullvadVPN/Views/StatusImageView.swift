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

class StatusImageView: UIImageView {
    enum Style: Int {
        case success
        case failure

        fileprivate var image: UIImage? {
            switch self {
            case .success:
                return UIImage.Status.success
            case .failure:
                return UIImage.Status.failure
            }
        }
    }

    var style: Style = .success {
        didSet {
            self.image = style.image
        }
    }

    override var accessibilityValue: String? {
        get {
            switch style {
            case .success:
                return "success"
            case .failure:
                return "fail"
            }
        }

        set {
            fatalError("This accessibilityValue property is get only")
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        image = style.image
    }

    init(style: Style) {
        self.style = style
        super.init(image: style.image)
        image = style.image
        self.adjustsImageSizeForAccessibilityContentSizeCategory = true
        setAccessibilityIdentifier(.statusImageView)
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }
}
