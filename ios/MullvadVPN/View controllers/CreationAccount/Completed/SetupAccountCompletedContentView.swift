//
//  SetupAccountCompletedContentView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
        label.text = NSLocalizedString("You’re all set!!", comment: "")
        return label
    }()

    private let commentLabel: UILabel = {
        let label = UILabel()

        let message = NSMutableAttributedString(string: NSLocalizedString(
            """
            Go ahead and start using the app to begin reclaiming your online privacy.
            To continue your journey as a privacy ninja, \
            visit our website to pick up other privacy-friendly habits and tools.
            """,
            comment: ""
        ))
        message.apply(paragraphStyle: .alert)

        label.attributedText = message
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = .zero

        return label
    }()

    private let privacyButton: AppButton = {
        let button = AppButton(style: .success)
        button.setAccessibilityIdentifier(.learnAboutPrivacyButton)
        let localizedString = NSLocalizedString("Learn about privacy", comment: "")
        button.setTitle(localizedString, for: .normal)
        button.setImage(UIImage(named: "IconExtlink")?.imageFlippedForRightToLeftLayoutDirection(), for: .normal)
        return button
    }()

    private let startButton: AppButton = {
        let button = AppButton(style: .success)
        button.setAccessibilityIdentifier(.startUsingTheAppButton)
        button.setTitle(NSLocalizedString("Start using the app", comment: ""), for: .normal)
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
        switch button.accessibilityIdentifier {
        case AccessibilityIdentifier.learnAboutPrivacyButton.asString:
            delegate?.didTapPrivacyButton(view: self, button: button)
        case AccessibilityIdentifier.startUsingTheAppButton.asString:
            delegate?.didTapStartingAppButton(view: self, button: button)
        default: return
        }
    }
}
