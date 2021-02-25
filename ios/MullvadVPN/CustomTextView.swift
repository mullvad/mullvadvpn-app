//
//  CustomTextView.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

private let kTextViewCornerRadius = CGFloat(4)

class CustomTextView: UITextView {

    var roundCorners: Bool = true {
        didSet {
            layer.cornerRadius = roundCorners ? kTextViewCornerRadius : 0
        }
    }

    /// Placeholder string
    var placeholder: String? {
        set {
            placeholderTextLabel.text = newValue
        }
        get {
            return placeholderTextLabel.text
        }
    }

    /// Placeholder text label
    private let placeholderTextLabel = UILabel()

    /// Placeholder label constraints
    private var placeholderConstraints = [NSLayoutConstraint]()

    override var textContainerInset: UIEdgeInsets {
        didSet {
            setNeedsUpdateConstraints()
        }
    }

    override var font: UIFont? {
        didSet {
            placeholderTextLabel.font = self.font ?? UIFont.preferredFont(forTextStyle: .body)
        }
    }

    /// Placeholder text inset derived from `textContainerInset`
    private var placeholderTextInset: UIEdgeInsets {
        var placeholderInset = textContainerInset

        // Offset the placeholder label to match with text view rendering.
        placeholderInset.top += 0.5

        return placeholderInset
    }

    override init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)

        placeholderTextLabel.textColor = UIColor.TextField.placeholderTextColor
        placeholderTextLabel.highlightedTextColor = UIColor.TextField.placeholderTextColor
        placeholderTextLabel.translatesAutoresizingMaskIntoConstraints = false
        placeholderTextLabel.numberOfLines = 0
        addSubview(placeholderTextLabel)

        // Create placeholder constraints
        placeholderConstraints = [
            placeholderTextLabel.topAnchor.constraint(equalTo: topAnchor),
            placeholderTextLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            placeholderTextLabel.trailingAnchor.constraint(equalTo: trailingAnchor)
        ]
        NSLayoutConstraint.activate(placeholderConstraints)

        // Set visual appearance
        textColor = UIColor.TextField.textColor
        layer.cornerRadius = kTextViewCornerRadius
        clipsToBounds = true

        // Set content padding
        contentInset = .zero
        textContainerInset = UIEdgeInsets(top: 12, left: 14, bottom: 12, right: 14)
        self.textContainer.lineFragmentPadding = 0

        // Handle placeholder visibility
        NotificationCenter.default.addObserver(
            forName: NSTextStorage.didProcessEditingNotification,
            object: textStorage,
            queue: OperationQueue.main) { [weak self] (note) in
                self?.updatePlaceholderVisibility()
        }

        updatePlaceholderVisibility()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func updateConstraints() {
        let textInset = placeholderTextInset

        for constraint in placeholderConstraints {
            switch constraint.firstAttribute {
            case .top:
                constraint.constant = textInset.top
            case .leading:
                constraint.constant = textInset.left
            case .trailing:
                constraint.constant = textInset.right
            default:
                break
            }
        }

        super.updateConstraints()
    }

    private func updatePlaceholderVisibility() {
        placeholderTextLabel.isHidden = textStorage.length > 0
    }

}
