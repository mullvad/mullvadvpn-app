//
//  IPOverrideHeaderView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-20.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Header view pinned at the top of ``IPOverrideViewController``.
class IPOverrideHeaderView: UIView, UITextViewDelegate {
    /// Event handler invoked when user taps on the link to learn more about API access.
    var onAbout: (() -> Void)?

    private let textView = UITextView()

    override init(frame: CGRect) {
        super.init(frame: frame)

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
        let body = NSLocalizedString(
            "IP_OVERRIDE_HEADER_BODY",
            tableName: "IPOverride",
            value: "Import files or text with new IP addresses for the servers in the Select location view.",
            comment: ""
        )
        let link = NSLocalizedString(
            "IP_OVERRIDE_HEADER_LINK",
            tableName: "IPOverride",
            value: "About IP override...",
            comment: ""
        )

        var linkAttributes = defaultLinkAttributes
        linkAttributes[.link] = "#"

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping

        let attributedString = NSMutableAttributedString()
        attributedString.append(NSAttributedString(string: body, attributes: defaultTextAttributes))
        attributedString.append(NSAttributedString(string: " ", attributes: defaultTextAttributes))
        attributedString.append(NSAttributedString(string: link, attributes: linkAttributes))
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
