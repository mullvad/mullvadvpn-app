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
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
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
        subheadLabel.translatesAutoresizingMaskIntoConstraints = false
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
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.backgroundColor = .clear
        textView.isEditable = false
        textView.textContainerInset = UIMetrics.contentLayoutMargins
        return textView
    }()

    private let okButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
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
        container.translatesAutoresizingMaskIntoConstraints = false
        container.layoutMargins = UIMetrics.contentLayoutMargins
        container.backgroundColor = .secondaryColor
        return container
    }()

    var didTapButton: (() -> Void)?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        layoutMargins = UIMetrics.contentLayoutMargins

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
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineHeightMultiple = 1.5
        paragraphStyle.lineBreakMode = .byWordWrapping

        textView.attributedText = NSAttributedString(
            string: string,
            attributes: [
                .paragraphStyle: paragraphStyle,
                .font: UIFont.systemFont(ofSize: 18),
                .foregroundColor: UIColor.white,
            ]
        )
    }

    private func addSubviews() {
        footerContainer.addSubview(okButton)
        footerContainer.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)

        for subview in [titleLabel, subheadLabel, textView, footerContainer] {
            addSubview(subview)
        }

        NSLayoutConstraint.activate {
            titleLabel.pinEdgesToMargins(self, excludingEdges: .bottom)

            subheadLabel.pinHorizontalEdgesToMargins(self)
            subheadLabel.topAnchor.constraint(
                equalToSystemSpacingBelow: titleLabel.bottomAnchor,
                multiplier: 1
            )

            textView.topAnchor.constraint(equalTo: subheadLabel.bottomAnchor)
            textView.pinHorizontalEdgesTo(self)

            footerContainer.pinEdgesTo(self, excludingEdges: .top)
            footerContainer.topAnchor.constraint(equalTo: textView.bottomAnchor)

            okButton.pinEdgesToMargins(footerContainer)
        }
    }

    @objc private func handleButtonTap() {
        didTapButton?()
    }
}
