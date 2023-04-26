//
//  ChangeLogContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class ChangeLogContentView: UIView {
    private let titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.font = .systemFont(ofSize: 24, weight: .bold)
        titleLabel.numberOfLines = 0
        titleLabel.textColor = .white
        titleLabel.allowsDefaultTighteningForTruncation = true
        titleLabel.lineBreakMode = .byWordWrapping
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            titleLabel.lineBreakStrategy = []
        }
        return titleLabel
    }()

    private let subheadLabel: UILabel = {
        let subheadLabel = UILabel()
        subheadLabel.font = .systemFont(ofSize: 18, weight: .bold)
        subheadLabel.numberOfLines = 0
        subheadLabel.textColor = .white
        subheadLabel.allowsDefaultTighteningForTruncation = true
        subheadLabel.lineBreakMode = .byWordWrapping
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            subheadLabel.lineBreakStrategy = []
        }
        subheadLabel.text = NSLocalizedString(
            "CHANGES_IN_THIS_VERSION",
            tableName: "ChangeLog",
            value: "Changes in this version:",
            comment: ""
        )
        return subheadLabel
    }()

    private let textView: UITextView = {
        let textView = UITextView()
        textView.backgroundColor = .clear
        textView.isEditable = false
        textView.isSelectable = false
        textView.textContainerInset = UIMetrics.contentInsets
        return textView
    }()

    private let okButton: AppButton = {
        let button = AppButton(style: .default)
        button.accessibilityIdentifier = "OkButton"
        button.setTitle(NSLocalizedString(
            "OK_BUTTON",
            tableName: "ChangeLog",
            value: "Got it",
            comment: ""
        ), for: .normal)
        return button
    }()

    private let footerContainer: UIView = {
        let container = UIView()
        container.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        container.backgroundColor = .secondaryColor
        return container
    }()

    var didTapButton: (() -> Void)?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins

        okButton.addTarget(self, action: #selector(handleButtonTap), for: .touchUpInside)

        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setApplicationVersion(_ string: String) {
        titleLabel.text = string
    }

    func setChangeLogText(_ string: String) {
        let bullet = "• "
        let font = UIFont.systemFont(ofSize: 18)

        let bulletList = string.split(whereSeparator: { $0.isNewline })
            .map { "\(bullet)\($0)" }
            .joined(separator: "\n")

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineHeightMultiple = 1.5
        paragraphStyle.lineBreakMode = .byWordWrapping
        paragraphStyle.headIndent = bullet.size(withAttributes: [.font: font]).width

        textView.attributedText = NSAttributedString(
            string: bulletList,
            attributes: [
                .paragraphStyle: paragraphStyle,
                .font: font,
                .foregroundColor: UIColor.white,
            ]
        )
    }

    private func addSubviews() {
        footerContainer.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)

        footerContainer.addConstrainedSubviews([okButton]) {
            okButton.pinEdgesToSuperviewMargins()
        }

        addConstrainedSubviews([titleLabel, subheadLabel, textView, footerContainer]) {
            titleLabel.pinEdgesToSuperviewMargins(.all().excluding(.bottom))
            subheadLabel.pinEdgesToSuperviewMargins(.init([.leading(0), .trailing(0)]))
            subheadLabel.topAnchor.constraint(
                equalToSystemSpacingBelow: titleLabel.bottomAnchor,
                multiplier: 1
            )

            textView.topAnchor.constraint(equalTo: subheadLabel.bottomAnchor)
            textView.pinEdgesToSuperview(.init([.leading(0), .trailing(0)]))

            footerContainer.pinEdgesToSuperview(.all().excluding(.top))
            footerContainer.topAnchor.constraint(equalTo: textView.bottomAnchor)
        }
    }

    @objc private func handleButtonTap() {
        didTapButton?()
    }
}
