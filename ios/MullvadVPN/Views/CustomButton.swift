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

extension UIControl.State {
    var customButtonTitleColor: UIColor? {
        switch self {
        case .normal:
            return UIColor.AppButton.normalTitleColor
        case .disabled:
            return UIColor.AppButton.disabledTitleColor
        case .highlighted:
            return UIColor.AppButton.highlightedTitleColor
        default:
            return nil
        }
    }
}

/// A custom `UIButton` subclass that implements additional layouts for the image
class CustomButton: UIButton, Sendable {
    var imageAlignment: NSDirectionalRectEdge = .leading {
        didSet {
            self.configuration?.imagePlacement = imageAlignment
        }
    }

    var inlineImageSpacing: CGFloat = 4 {
        didSet {
            self.configuration?.imagePadding = inlineImageSpacing
        }
    }

    var titleAlignment: UIButton.Configuration.TitleAlignment = .center {
        didSet {
            self.configuration?.titleAlignment = titleAlignment
        }
    }

    var inlineTitleSpacing: CGFloat = 4 {
        didSet {
            self.configuration?.titlePadding = inlineTitleSpacing
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    private func commonInit() {
        var config = UIButton.Configuration.plain()
        config.imagePadding = inlineImageSpacing
        config.imagePlacement = imageAlignment
        config.titleAlignment = titleAlignment
        config.titleLineBreakMode = .byWordWrapping
        config.titlePadding = inlineTitleSpacing
        config.baseForegroundColor = state.customButtonTitleColor
        self.configuration = config
    }
}
