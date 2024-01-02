//
//  ListAccessMethodHeaderView.swift
//  MullvadVPN
//
//  Created by pronebird on 07/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Header view pinned at the top of ``AccessMethodListViewController``.
class ListAccessMethodHeaderView: UIView, UITextViewDelegate {
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

        directionalLayoutMargins = UIMetrics.contentHeadingLayoutMargins

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
            "ACCESS_METHOD_HEADER_BODY",
            tableName: "APIAccess",
            value: "Manage default and setup custom methods to access the Mullvad API.",
            comment: ""
        )
        let link = NSLocalizedString(
            "ACCESS_METHOD_HEADER_LINK",
            tableName: "APIAccess",
            value: "About API access...",
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
