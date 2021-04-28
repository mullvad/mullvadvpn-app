//
//  ConsentContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 28/04/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ConsentContentView: UIView {

    let logoImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "LogoIcon"))
        imageView.translatesAutoresizingMaskIntoConstraints = false
        return imageView
    }()

    let titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 24, weight: .bold)
        titleLabel.numberOfLines = 0
        titleLabel.textColor = .white
        titleLabel.allowsDefaultTighteningForTruncation = true
        titleLabel.text = NSLocalizedString("Do you agree to remaining anonymous?", comment: "")
        titleLabel.lineBreakMode = .byWordWrapping
        if #available(iOS 14.0, *) {
            // Disable the new line break strategy used by UIKit that moves at least two words
            // to the next line which makes the title look odd.
            // See: https://stackoverflow.com/q/46200027/351305
            titleLabel.lineBreakStrategy = []
        }
        return titleLabel
    }()

    let bodyLabel: UILabel = {
        let localizedText = NSLocalizedString("""
You have a right to privacy. That’s why we never store activity logs, don't ask for personal information, and encourage anonymous payments.

In some situations, as outlined in our privacy policy, we might process personal data that you choose to send, for example if you email us.

We strongly believe in retaining as little data as possible because we want you to remain anonymous.
""", comment: "")

        let bodyLabel = UILabel()
        bodyLabel.translatesAutoresizingMaskIntoConstraints = false
        bodyLabel.font = UIFont.systemFont(ofSize: 18)
        bodyLabel.textColor = .white
        bodyLabel.numberOfLines = 0
        bodyLabel.text = localizedText
        return bodyLabel
    }()

    let privacyPolicyLink: LinkButton = {
        let button = LinkButton()
        button.translatesAutoresizingMaskIntoConstraints = false
        button.titleString = NSLocalizedString("Privacy policy", comment: "")
        button.setImage(UIImage(named: "IconExtlink"), for: .normal)
        return button
    }()

    let agreeButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString("Agree and continue", comment: ""), for: .normal)
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

    private var logoImageVisiblTitleTopConstraint: NSLayoutConstraint?
    private var logoImageHiddenTitleTopConstraint: NSLayoutConstraint?

    override init(frame: CGRect) {
        super.init(frame: frame)

        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if previousTraitCollection?.userInterfaceIdiom != traitCollection.userInterfaceIdiom ||
            previousTraitCollection?.horizontalSizeClass != traitCollection.horizontalSizeClass {
            updateTraitConstraints()
        }
    }

    // MARK: - Private

    private func addSubviews() {
        addSubview(scrollView)
        addSubview(footerContainer)

        scrollView.addSubview(scrollContentContainer)
        [logoImageView, titleLabel, bodyLabel, privacyPolicyLink].forEach { scrollContentContainer.addSubview($0) }
        footerContainer.addSubview(agreeButton)

        scrollView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)
        footerContainer.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)

        logoImageVisiblTitleTopConstraint = titleLabel.topAnchor.constraint(equalTo: logoImageView.bottomAnchor, constant: 20)
        logoImageHiddenTitleTopConstraint = titleLabel.topAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.topAnchor)

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
            agreeButton.leadingAnchor.constraint(equalTo: footerContainer.layoutMarginsGuide.leadingAnchor),
            agreeButton.trailingAnchor.constraint(equalTo: footerContainer.layoutMarginsGuide.trailingAnchor),
            agreeButton.bottomAnchor.constraint(equalTo: footerContainer.layoutMarginsGuide.bottomAnchor),

            logoImageView.topAnchor.constraint(equalTo: scrollContentContainer.topAnchor, constant: 20),
            logoImageView.centerXAnchor.constraint(equalTo: scrollContentContainer.centerXAnchor),
            logoImageView.widthAnchor.constraint(equalToConstant: 60),
            logoImageView.heightAnchor.constraint(equalTo: logoImageView.widthAnchor),

            logoImageVisiblTitleTopConstraint!,
            titleLabel.leadingAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.trailingAnchor),

            bodyLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 24),
            bodyLabel.leadingAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.leadingAnchor),
            bodyLabel.trailingAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.trailingAnchor),

            privacyPolicyLink.topAnchor.constraint(equalTo: bodyLabel.bottomAnchor, constant: 24),
            privacyPolicyLink.leadingAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.leadingAnchor),
            privacyPolicyLink.trailingAnchor.constraint(lessThanOrEqualTo: scrollContentContainer.layoutMarginsGuide.trailingAnchor),
            privacyPolicyLink.bottomAnchor.constraint(equalTo: scrollContentContainer.layoutMarginsGuide.bottomAnchor),

        ])
    }

    private func updateTraitConstraints() {
        switch (traitCollection.userInterfaceIdiom, traitCollection.horizontalSizeClass) {
        case (.pad, .compact):
            // Hide logo on iPad in compact mode as we show the header bar that already contains the logo
            logoImageView.isHidden = true
            logoImageVisiblTitleTopConstraint?.isActive = false
            logoImageHiddenTitleTopConstraint?.isActive = true

        default:
            logoImageView.isHidden = false
            logoImageHiddenTitleTopConstraint?.isActive = false
            logoImageVisiblTitleTopConstraint?.isActive = true
        }
    }

}
