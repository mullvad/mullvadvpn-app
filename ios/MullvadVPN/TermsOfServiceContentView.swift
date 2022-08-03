//
//  TermsOfServiceContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 28/04/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class TermsOfServiceContentView: UIView {
    let titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 24, weight: .bold)
        titleLabel.numberOfLines = 0
        titleLabel.textColor = .white
        titleLabel.allowsDefaultTighteningForTruncation = true
        titleLabel.text = NSLocalizedString(
            "PRIVACY_NOTICE_HEADING",
            tableName: "TermsOfService",
            value: "Do you agree to remaining anonymous?",
            comment: ""
        )
        titleLabel.lineBreakMode = .byWordWrapping
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            titleLabel.lineBreakStrategy = []
        }
        return titleLabel
    }()

    let bodyLabel: UILabel = {
        let bodyLabel = UILabel()
        bodyLabel.translatesAutoresizingMaskIntoConstraints = false
        bodyLabel.font = UIFont.systemFont(ofSize: 18)
        bodyLabel.textColor = .white
        bodyLabel.numberOfLines = 0
        bodyLabel.text = NSLocalizedString(
            "PRIVACY_NOTICE_BODY",
            tableName: "TermsOfService",
            value: "You have a right to privacy. That’s why we never store activity logs, don't ask for personal information, and encourage anonymous payments.\n\nIn some situations, as outlined in our privacy policy, we might process personal data that you choose to send, for example if you email us.\n\nWe strongly believe in retaining as little data as possible because we want you to remain anonymous.",
            comment: ""
        )
        return bodyLabel
    }()

    let privacyPolicyLink: LinkButton = {
        let button = LinkButton()
        button.translatesAutoresizingMaskIntoConstraints = false
        button.titleString = NSLocalizedString(
            "PRIVACY_POLICY_LINK_TITLE",
            tableName: "TermsOfService",
            value: "Privacy policy",
            comment: ""
        )
        button.setImage(UIImage(named: "IconExtlink"), for: .normal)
        return button
    }()

    let agreeButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.accessibilityIdentifier = "AgreeButton"
        button.setTitle(NSLocalizedString(
            "CONTINUE_BUTTON_TITLE",
            tableName: "TermsOfService",
            value: "Agree and continue",
            comment: ""
        ), for: .normal)
        return button
    }()

    let scrollView: UIScrollView = {
        let scrollView = UIScrollView()
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        return scrollView
    }()

    let scrollContentContainer: UIView = {
        let contentView = UIView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        contentView.layoutMargins = UIMetrics.contentLayoutMargins
        return contentView
    }()

    let footerContainer: UIView = {
        let container = UIView()
        container.translatesAutoresizingMaskIntoConstraints = false
        container.layoutMargins = UIMetrics.contentLayoutMargins
        container.backgroundColor = .secondaryColor
        return container
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Private

    private func addSubviews() {
        addSubview(scrollView)
        addSubview(footerContainer)

        scrollView.addSubview(scrollContentContainer)
        [titleLabel, bodyLabel, privacyPolicyLink].forEach { scrollContentContainer.addSubview($0) }
        footerContainer.addSubview(agreeButton)

        scrollView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)
        footerContainer.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)

        NSLayoutConstraint.activate([
            scrollView.topAnchor.constraint(equalTo: safeAreaLayoutGuide.topAnchor),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor),

            scrollContentContainer.widthAnchor.constraint(equalTo: scrollView.widthAnchor),
            scrollContentContainer.topAnchor.constraint(equalTo: scrollView.topAnchor),
            scrollContentContainer.leadingAnchor.constraint(equalTo: scrollView.leadingAnchor),
            scrollContentContainer.trailingAnchor.constraint(equalTo: scrollView.trailingAnchor),
            scrollContentContainer.bottomAnchor.constraint(equalTo: scrollView.bottomAnchor),

            footerContainer.topAnchor.constraint(equalTo: scrollView.bottomAnchor),
            footerContainer.leadingAnchor.constraint(equalTo: leadingAnchor),
            footerContainer.trailingAnchor.constraint(equalTo: trailingAnchor),
            footerContainer.bottomAnchor.constraint(equalTo: bottomAnchor),

            agreeButton.topAnchor.constraint(equalTo: footerContainer.layoutMarginsGuide.topAnchor),
            agreeButton.leadingAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.leadingAnchor),
            agreeButton.trailingAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.trailingAnchor),
            agreeButton.bottomAnchor
                .constraint(equalTo: footerContainer.layoutMarginsGuide.bottomAnchor),

            titleLabel.topAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.topAnchor),
            titleLabel.leadingAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.trailingAnchor),

            bodyLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 24),
            bodyLabel.leadingAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.leadingAnchor),
            bodyLabel.trailingAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.trailingAnchor),

            privacyPolicyLink.topAnchor.constraint(equalTo: bodyLabel.bottomAnchor, constant: 24),
            privacyPolicyLink.leadingAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.leadingAnchor),
            privacyPolicyLink.trailingAnchor
                .constraint(
                    lessThanOrEqualTo: scrollContentContainer.layoutMarginsGuide
                        .trailingAnchor
                ),
            privacyPolicyLink.bottomAnchor
                .constraint(equalTo: scrollContentContainer.layoutMarginsGuide.bottomAnchor),

        ])
    }
}
