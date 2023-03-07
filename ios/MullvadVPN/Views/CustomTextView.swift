//
//  CustomTextView.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomTextView: UITextView {
    private static let textViewCornerRadius: CGFloat = 4

    var roundCorners = true {
        didSet {
            layer.cornerRadius = roundCorners ? Self.textViewCornerRadius : 0
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
            placeholderTextLabel.font = font ?? UIFont.preferredFont(forTextStyle: .body)
        }
    }

    /// Placeholder text inset derived from `textContainerInset`
    private var placeholderTextInset: UIEdgeInsets {
        var placeholderInset = textContainerInset

        // Offset the placeholder label to match with text view rendering.
        placeholderInset.top += 0.5

        return placeholderInset
    }

    override var accessibilityLabel: String? {
        set {
            super.accessibilityLabel = newValue
        }
        get {
            if self.text.isEmpty {
                return placeholderTextLabel.text
            } else {
                return super.accessibilityLabel
            }
        }
    }

    override var accessibilityPath: UIBezierPath? {
        set {
            super.accessibilityPath = newValue
        }
        get {
            if roundCorners {
                return UIBezierPath(
                    roundedRect: accessibilityFrame,
                    cornerRadius: Self.textViewCornerRadius
                )
            } else {
                return UIBezierPath(rect: accessibilityFrame)
            }
        }
    }

    private var notificationObserver: Any?

    override init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)

        placeholderTextLabel.isAccessibilityElement = false
        placeholderTextLabel.accessibilityTraits = []
        placeholderTextLabel.textColor = UIColor.TextField.placeholderTextColor
        placeholderTextLabel.highlightedTextColor = UIColor.TextField.placeholderTextColor
        placeholderTextLabel.translatesAutoresizingMaskIntoConstraints = false
        placeholderTextLabel.numberOfLines = 0
        addSubview(placeholderTextLabel)

        // Create placeholder constraints
        placeholderConstraints = [
            placeholderTextLabel.topAnchor.constraint(equalTo: safeAreaLayoutGuide.topAnchor),
            placeholderTextLabel.leadingAnchor
                .constraint(equalTo: safeAreaLayoutGuide.leadingAnchor),
            placeholderTextLabel.trailingAnchor
                .constraint(equalTo: safeAreaLayoutGuide.trailingAnchor),
            placeholderTextLabel.bottomAnchor
                .constraint(lessThanOrEqualTo: safeAreaLayoutGuide.bottomAnchor),
        ]
        NSLayoutConstraint.activate(placeholderConstraints)

        // Set visual appearance
        textColor = UIColor.TextField.textColor
        layer.cornerRadius = Self.textViewCornerRadius
        clipsToBounds = true

        // Set content padding
        contentInset = .zero
        textContainerInset = UIEdgeInsets(top: 12, left: 14, bottom: 12, right: 14)
        self.textContainer.lineFragmentPadding = 0

        // Handle placeholder visibility
        notificationObserver = NotificationCenter.default.addObserver(
            forName: NSTextStorage.didProcessEditingNotification,
            object: textStorage,
            queue: OperationQueue.main
        ) { [weak self] note in
            self?.updatePlaceholderVisibility()
        }

        updatePlaceholderVisibility()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    deinit {
        if let notificationObserver = notificationObserver {
            NotificationCenter.default.removeObserver(notificationObserver)
        }
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
                constraint.constant = -textInset.right
            case .bottom:
                constraint.constant = -textInset.bottom
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
