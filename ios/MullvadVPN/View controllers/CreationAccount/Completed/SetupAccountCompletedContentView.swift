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
        label.text = NSLocalizedString("You’re all set!", comment: "")
        return label
    }()

    private let commentLabel: UILabel = {
        let label = UILabel()

        let message = NSMutableAttributedString(
            string: [
                NSLocalizedString(
                    "Go ahead and start using the app to begin reclaiming your online privacy.",
                    comment: ""
                ),
                NSLocalizedString(
                    "To continue your journey as a privacy ninja, visit our website to pick up"
                        + " other privacy-friendly habits and tools.",
                    comment: ""
                ),
            ].joined(separator: " "))
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

    let scrollView = UIScrollView()

    weak var delegate: SetupAccountCompletedContentViewDelegate?

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        backgroundColor = .secondaryColor

        setAccessibilityIdentifier(.setUpAccountCompletedView)

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

        scrollView.addConstrainedSubviews([textsStackView]) {
            textsStackView.pinEdgesToSuperviewMargins(
                PinnableEdges([
                    .leading(0),
                    .trailing(0),
                ]))

            textsStackView.pinEdgesToSuperview(
                PinnableEdges([
                    .top(0),
                    .bottom(0),
                ]))
        }

        addConstrainedSubviews([scrollView, buttonsStackView]) {
            scrollView.pinEdgesToSuperviewMargins(
                PinnableEdges([
                    .top(UIMetrics.contentLayoutMargins.top),
                    .leading(0),
                    .trailing(0),
                ]))

            buttonsStackView.pinEdgesToSuperviewMargins(
                PinnableEdges([
                    .leading(UIMetrics.padding8),
                    .trailing(UIMetrics.padding8),
                    .bottom(UIMetrics.contentLayoutMargins.bottom),
                ]))

            buttonsStackView.topAnchor.constraint(
                equalTo: scrollView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            )
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
