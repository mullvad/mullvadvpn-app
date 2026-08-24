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

class DynamicImageView: UIImageView {
    private let baseSize: CGFloat
    private let textStyle: UIFont.TextStyle

    init(image: UIImage?, baseSize: CGFloat = 24.0, textStyle: UIFont.TextStyle = .body) {
        self.baseSize = baseSize
        self.textStyle = textStyle
        super.init(image: image)
        registerForTraitChanges([UITraitPreferredContentSizeCategory.self]) {
            (self: Self, previousTraitCollection: UITraitCollection) in
            self.invalidateIntrinsicContentSize()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var intrinsicContentSize: CGSize {
        let scaledSize = UIFontMetrics(forTextStyle: textStyle).scaledValue(for: baseSize)
        return CGSize(width: scaledSize, height: scaledSize)
    }
}
