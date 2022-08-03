//
//  CustomTextField.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

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
        return bounds.inset(by: textMargins)
    }

    override func editingRect(forBounds bounds: CGRect) -> CGRect {
        return textRect(forBounds: bounds)
    }

    private func updatePlaceholderTextColor() {
        for case let placeholderLabel as UILabel in subviews {
            placeholderLabel.textColor = placeholderTextColor
            break
        }
    }
}
