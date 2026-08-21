// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import UIKit

class CustomTextField: UITextField {
    var cornerRadius: CGFloat = UIMetrics.controlCornerRadius {
        didSet {
            layer.cornerRadius = cornerRadius
        }
    }

    var textMargins = UIMetrics.textFieldMargins {
        didSet {
            setNeedsLayout()
        }
    }

    var placeholderTextColor = UIColor.TextField.placeholderTextColor {
        didSet {
            updatePlaceholderTextColor()
        }
    }

    override var placeholder: String? {
        didSet {
            updatePlaceholderTextColor()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        textColor = UIColor.TextField.textColor
        layer.cornerRadius = cornerRadius
        clipsToBounds = true
    }

    override func didAddSubview(_ subview: UIView) {
        super.didAddSubview(subview)

        // Internally `UITextField` adds the placeholder label to its view hierarchy.
        // Intercept it here and update the text color.
        if let placeholderLabel = subview as? UILabel {
            placeholderLabel.textColor = placeholderTextColor
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func textRect(forBounds bounds: CGRect) -> CGRect {
        bounds.inset(by: textMargins)
    }

    override func editingRect(forBounds bounds: CGRect) -> CGRect {
        textRect(forBounds: bounds)
    }

    private func updatePlaceholderTextColor() {
        for case let placeholderLabel as UILabel in subviews {
            placeholderLabel.textColor = placeholderTextColor
            break
        }
    }
}
