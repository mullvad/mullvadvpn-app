//
//  GrowingTextEditor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI
import UIKit

struct GrowingTextEditor: UIViewRepresentable {
    typealias UIViewType = UITextView

    var placeholder: String
    @Binding var text: String
    @Binding var isFocused: Bool

    var maxHeight: CGFloat = 66.0
    var font: UIFont = .mullvadSmall
    var foregroundColor: Color = .mullvadTextPrimary
    var placeholderColor: Color = .mullvadTextSecondary
    var accessibilityIdentifier: AccessibilityIdentifier?

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UITextView {
        let textView = PlaceholderTextView()
        textView.delegate = context.coordinator
        textView.font = font
        textView.backgroundColor = .clear
        textView.textContainerInset = .init(top: 8.0, left: 8.0, bottom: 8.0, right: 8.0)
        textView.textContainer.lineFragmentPadding = 0
        textView.textColor = UIColor(foregroundColor)
        textView.placeholder = placeholder
        textView.placeholderColor = UIColor(placeholderColor)
        textView.accessibilityIdentifier = accessibilityIdentifier?.asString
        textView.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        textView.isScrollEnabled = false

        if isFocused && !textView.isFirstResponder {
            textView.becomeFirstResponder()
        } else if !isFocused && textView.isFirstResponder {
            textView.resignFirstResponder()
        }

        return textView
    }

    func updateUIView(_ uiView: UITextView, context: Context) {
        if uiView.text != text {
            uiView.text = text
        }
        context.coordinator.update(uiView)
    }

    internal class Coordinator: NSObject, UITextViewDelegate {
        var parent: GrowingTextEditor

        init(parent: GrowingTextEditor) {
            self.parent = parent
        }

        func textViewDidChange(_ textView: UITextView) {
            parent.text = textView.text
            update(textView)
        }

        func textViewDidBeginEditing(_ textView: UITextView) {
            parent.isFocused = true
        }

        func textViewDidEndEditing(_ textView: UITextView) {
            parent.isFocused = false
        }

        func update(_ textView: UITextView) {
            let size = textView.sizeThatFits(
                CGSize(
                    width: textView.bounds.width,
                    height: .greatestFiniteMagnitude)
            )

            let shouldScroll = size.height > parent.maxHeight

            if textView.isScrollEnabled != shouldScroll {
                textView.isScrollEnabled = shouldScroll
            }

            if shouldScroll {
                textView.scrollRangeToVisible(textView.selectedRange)
            }
        }
    }
}

private class PlaceholderTextView: UITextView {

    private let placeholderLabel = UILabel()

    override init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        setup()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setup()
    }

    var placeholder: String = "" {
        didSet {
            placeholderLabel.text = placeholder
        }
    }

    var placeholderColor: UIColor = .secondaryTextColor {
        didSet {
            placeholderLabel.textColor = placeholderColor
        }
    }

    override var font: UIFont? {
        didSet {
            placeholderLabel.font = font
        }
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        placeholderLabel.frame.origin = CGPoint(
            x: textContainerInset.left + textContainer.lineFragmentPadding,
            y: textContainerInset.top
        )

        placeholderLabel.preferredMaxLayoutWidth =
            bounds.width
            - textContainerInset.left
            - textContainerInset.right
            - textContainer.lineFragmentPadding * 2

        placeholderLabel.sizeToFit()
    }

    private func setup() {
        placeholderLabel.textColor = .placeholderText
        placeholderLabel.numberOfLines = 0
        addSubview(placeholderLabel)

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextView.textDidChangeNotification,
            object: self
        )
    }

    @objc private func textDidChange() {
        updatePlaceholderVisibility()
    }

    private func updatePlaceholderVisibility() {
        placeholderLabel.isHidden = !text.isEmpty
    }
}
