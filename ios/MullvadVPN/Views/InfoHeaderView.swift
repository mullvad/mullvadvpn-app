//
//  InfoHeaderView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

/// Header view pinned at the top of a ``ViewController``.
class InfoHeaderView: UIView, UITextViewDelegate {
    /// Event handler invoked when user taps on the link.
    var onAbout: (() -> Void)?

    private let infoLabel = UILabel()
    private let config: InfoHeaderConfig

    init(config: InfoHeaderConfig) {
        self.config = config

        super.init(frame: .zero)

        infoLabel.backgroundColor = .clear
        infoLabel.attributedText = makeAttributedString()
        infoLabel.numberOfLines = 0
        infoLabel.accessibilityTraits = .link

        directionalLayoutMargins = .zero

        addSubviews()
        addTapGestureRecognizer()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private let defaultTextAttributes: [NSAttributedString.Key: Any] = [
        .font: UIFont.systemFont(ofSize: 13),
        .foregroundColor: UIColor.ContentHeading.textColor,
    ]

    private let defaultLinkAttributes: [NSAttributedString.Key: Any] = [
        .font: UIFont.boldSystemFont(ofSize: 13),
        .foregroundColor: UIColor.ContentHeading.linkColor,
        .attachment: "#",
    ]

    private func makeAttributedString() -> NSAttributedString {
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping

        let attributedString = NSMutableAttributedString()
        attributedString.append(NSAttributedString(string: config.body, attributes: defaultTextAttributes))
        attributedString.append(NSAttributedString(string: " ", attributes: defaultTextAttributes))
        attributedString.append(NSAttributedString(string: config.link, attributes: defaultLinkAttributes))
        attributedString.addAttribute(
            .paragraphStyle,
            value: paragraphStyle,
            range: NSRange(location: 0, length: attributedString.length)
        )
        return attributedString
    }

    private func addSubviews() {
        addConstrainedSubviews([infoLabel]) {
            infoLabel.pinEdgesToSuperviewMargins()
        }
    }

    private func addTapGestureRecognizer() {
        let tapGesture = UITapGestureRecognizer(target: self, action: #selector(handleTextViewTap))
        addGestureRecognizer(tapGesture)
    }

    @objc
    private func handleTextViewTap() {
        onAbout?()
    }
}
