//
//  InfoHeaderView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-01.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

/// Header view pinned at the top of a ``ViewController``.
class InfoHeaderView: UIView, UITextViewDelegate {
    /// Event handler invoked when user taps on the link.
    var onAbout: (() -> Void)?

    private let textView = UITextView()
    private let config: InfoHeaderConfig

    init(config: InfoHeaderConfig) {
        self.config = config

        super.init(frame: .zero)

        textView.backgroundColor = .clear
        textView.dataDetectorTypes = .link
        textView.isSelectable = true
        textView.isEditable = false
        textView.isScrollEnabled = false
        textView.contentInset = .zero
        textView.textContainerInset = .zero
        textView.attributedText = makeAttributedString()
        textView.linkTextAttributes = defaultLinkAttributes
        textView.textContainer.lineFragmentPadding = 0
        textView.delegate = self

        directionalLayoutMargins = .zero

        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private let defaultTextAttributes: [NSAttributedString.Key: Any] = [
        .font: UIFont.systemFont(ofSize: 13),
        .foregroundColor: UIColor.ContentHeading.textColor,
    ]

    private let defaultLinkAttributes: [NSAttributedString.Key: Any] = [
        .font: UIFont.systemFont(ofSize: 13),
        .foregroundColor: UIColor.ContentHeading.linkColor,
    ]

    private func makeAttributedString() -> NSAttributedString {
        var linkAttributes = defaultLinkAttributes
        linkAttributes[.link] = "#"

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping

        let attributedString = NSMutableAttributedString()
        attributedString.append(NSAttributedString(string: config.body, attributes: defaultTextAttributes))
        attributedString.append(NSAttributedString(string: " ", attributes: defaultTextAttributes))
        attributedString.append(NSAttributedString(string: config.link, attributes: linkAttributes))
        attributedString.addAttribute(
            .paragraphStyle,
            value: paragraphStyle,
            range: NSRange(location: 0, length: attributedString.length)
        )
        return attributedString
    }

    private func addSubviews() {
        addConstrainedSubviews([textView]) {
            textView.pinEdgesToSuperviewMargins()
        }
    }

    func textView(
        _ textView: UITextView,
        shouldInteractWith URL: URL,
        in characterRange: NSRange,
        interaction: UITextItemInteraction
    ) -> Bool {
        onAbout?()
        return false
    }

    @available(iOS 17.0, *)
    func textView(_ textView: UITextView, menuConfigurationFor textItem: UITextItem, defaultMenu: UIMenu) -> UITextItem
        .MenuConfiguration? {
        return nil
    }

    @available(iOS 17.0, *)
    func textView(_ textView: UITextView, primaryActionFor textItem: UITextItem, defaultAction: UIAction) -> UIAction? {
        if case .link = textItem.content {
            return UIAction { [weak self] _ in
                self?.onAbout?()
            }
        }
        return nil
    }
}
