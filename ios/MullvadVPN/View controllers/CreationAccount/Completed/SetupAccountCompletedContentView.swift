//
//  SetupAccountCompletedContentView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol SetupAccountCompletedContentViewDelegate: AnyObject {
    func didTapPrivacyButton(view: SetupAccountCompletedContentView, button: AppButton)
    func didTapStartingAppButton(view: SetupAccountCompletedContentView, button: AppButton)
}

class SetupAccountCompletedContentView: UIView {
    private let titleLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .largeTitle, weight: .bold)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = .zero
        label.text = NSLocalizedString(
            "CREATED_ACCOUNT_CONFIRMATION_PAGE_TITLE",
            tableName: "CreatedAccountConfirmation",
            value: "You’re all set!!",
            comment: ""
        )
        return label
    }()

    private let commentLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = .zero
        label.text = NSLocalizedString(
            "CREATED_ACCOUNT_CONFIRMATION_PAGE_BODY",
            tableName: "CreatedAccountConfirmation",
            value: """
            Go ahead and start using the app to begin reclaiming your online privacy.

            To continue your journey as a privacy ninja, \
            visit our website to pick up other privacy-friendly habits and tools.
            """,
            comment: ""
        )
        return label
    }()

    private let privacyButton: AppButton = {
        let button = AppButton(style: .success)
        button.accessibilityIdentifier = .learnAboutPrivacyButton
        let localizedString = NSLocalizedString(
            "LEARN_ABOUT_PRIVACY_BUTTON",
            tableName: "CreatedAccountConfirmation",
            value: "Learn about privacy",
            comment: ""
        )
        button.setTitle(localizedString, for: .normal)
        button.setImage(UIImage(named: "IconExtlink")?.imageFlippedForRightToLeftLayoutDirection(), for: .normal)
        return button
    }()

    private let startButton: AppButton = {
        let button = AppButton(style: .success)
        button.accessibilityIdentifier = .startUsingTheAppButton
        button.setTitle(NSLocalizedString(
            "START_USING_THE_APP_BUTTON",
            tableName: "CreatedAccountConfirmation",
            value: "Start using the app",
            comment: ""
        ), for: .normal)
        return button
    }()

    private let textsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .vertical
        return stackView
    }()

    private let buttonsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.interButtonSpacing
        return stackView
    }()

    weak var delegate: SetupAccountCompletedContentViewDelegate?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
        backgroundColor = .secondaryColor

        configureUI()
        addActions()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func configureUI() {
        textsStackView.addArrangedSubview(titleLabel)
        textsStackView.setCustomSpacing(UIMetrics.padding8, after: titleLabel)
        textsStackView.addArrangedSubview(commentLabel)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: commentLabel)

        buttonsStackView.addArrangedSubview(privacyButton)
        buttonsStackView.addArrangedSubview(startButton)

        addSubview(textsStackView)
        addSubview(buttonsStackView)
        addConstraints()
    }

    private func addConstraints() {
        addConstrainedSubviews([textsStackView, buttonsStackView]) {
            textsStackView
                .pinEdgesToSuperviewMargins(.all().excluding(.bottom))

            buttonsStackView
                .pinEdgesToSuperviewMargins(.all().excluding(.top))
        }
    }

    private func addActions() {
        [privacyButton, startButton].forEach {
            $0.addTarget(self, action: #selector(tapped(button:)), for: .touchUpInside)
        }
    }

    @objc private func tapped(button: AppButton) {
        switch AccessibilityIdentifier(rawValue: button.accessibilityIdentifier ?? "") {
        case .learnAboutPrivacyButton:
            delegate?.didTapPrivacyButton(view: self, button: button)
        case .startUsingTheAppButton:
            delegate?.didTapStartingAppButton(view: self, button: button)
        default: return
        }
    }
}
