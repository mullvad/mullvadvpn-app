//
//  WelcomeContentView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol WelcomeContentViewDelegate: AnyObject, Sendable {
    func didTapPurchaseButton(welcomeContentView: WelcomeContentView, button: AppButton)
    func didTapInfoButton(welcomeContentView: WelcomeContentView, button: UIButton)
    func didTapCopyButton(welcomeContentView: WelcomeContentView, button: UIButton)
}

struct WelcomeViewModel: Sendable {
    let deviceName: String
    let accountNumber: String
}

final class WelcomeContentView: UIView, Sendable {
    private var revertCopyImageWorkItem: DispatchWorkItem?

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .largeTitle, weight: .bold)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = .zero
        label.text = NSLocalizedString(
            "WELCOME_PAGE_TITLE",
            tableName: "Welcome",
            value: "Congrats!",
            comment: ""
        )
        return label
    }()

    private let subtitleLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = .zero
        label.text = NSLocalizedString(
            "WELCOME_PAGE_SUBTITLE",
            tableName: "Welcome",
            value: "Here’s your account number. Save it!",
            comment: ""
        )
        return label
    }()

    private let accountNumberLabel: UILabel = {
        let label = UILabel()
        label.setAccessibilityIdentifier(.welcomeAccountNumberLabel)
        label.adjustsFontForContentSizeCategory = true
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = .zero
        label.font = .preferredFont(forTextStyle: .title2, weight: .bold)
        label.textColor = .white
        return label
    }()

    private let copyButton: UIButton = {
        let button = UIButton(type: .system)
        button.setAccessibilityIdentifier(.copyButton)
        button.tintColor = .white
        button.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return button
    }()

    private let deviceNameLabel: UILabel = {
        let label = UILabel()
        label.adjustsFontForContentSizeCategory = true
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .white
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        return label
    }()

    private let infoButton: UIButton = {
        let button = IncreasedHitButton(type: .system)
        button.setAccessibilityIdentifier(.infoButton)
        button.tintColor = .white
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setImage(UIImage(named: "IconInfo"), for: .normal)
        button.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        button.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return button
    }()

    private let descriptionLabel: UILabel = {
        let label = UILabel()
        label.font = .preferredFont(forTextStyle: .body)
        label.adjustsFontForContentSizeCategory = true
        label.textColor = .white
        label.numberOfLines = .zero
        label.lineBreakMode = .byWordWrapping
        label.lineBreakStrategy = []
        label.text = NSLocalizedString(
            "WELCOME_PAGE_DESCRIPTION",
            tableName: "Welcome",
            value: """
            To start using the app, you first need to \
            add time to your account. Either buy credit \
            on our website or redeem a voucher.
            """,
            comment: ""
        )
        return label
    }()

    private let purchaseButton: InAppPurchaseButton = {
        let button = InAppPurchaseButton()
        button.setAccessibilityIdentifier(.purchaseButton)
        let localizedString = NSLocalizedString(
            "ADD_TIME_BUTTON",
            tableName: "Welcome",
            value: "Add time",
            comment: ""
        )
        button.setTitle(localizedString, for: .normal)
        return button
    }()

    private let textsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .vertical
        return stackView
    }()

    private let accountRowStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .horizontal
        stackView.distribution = .fill
        stackView.spacing = UIMetrics.padding8
        return stackView
    }()

    private let deviceRowStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .horizontal
        stackView.distribution = .fill
        return stackView
    }()

    private let spacerView: UIView = {
        let view = UIView()
        view.setContentHuggingPriority(.required, for: .horizontal)
        view.setContentCompressionResistancePriority(.required, for: .horizontal)
        return view
    }()

    private let buttonsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.interButtonSpacing
        return stackView
    }()

    weak var delegate: WelcomeContentViewDelegate?
    var viewModel: WelcomeViewModel? {
        didSet {
            accountNumberLabel.text = viewModel?.accountNumber
            deviceNameLabel.text = String(format: NSLocalizedString(
                "DEVICE_NAME_TEXT",
                tableName: "Welcome",
                value: "Device name: %@",
                comment: ""
            ), viewModel?.deviceName ?? "")
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        setAccessibilityIdentifier(.welcomeView)
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
        accountRowStackView.addArrangedSubview(accountNumberLabel)
        accountRowStackView.addArrangedSubview(copyButton)
        accountRowStackView.addArrangedSubview(UIView()) // To push content to the left.

        textsStackView.addArrangedSubview(titleLabel)
        textsStackView.setCustomSpacing(UIMetrics.padding8, after: titleLabel)
        textsStackView.addArrangedSubview(subtitleLabel)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: subtitleLabel)
        textsStackView.addArrangedSubview(accountRowStackView)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: accountRowStackView)

        deviceRowStackView.addArrangedSubview(deviceNameLabel)
        deviceRowStackView.setCustomSpacing(UIMetrics.padding8, after: deviceNameLabel)
        deviceRowStackView.addArrangedSubview(infoButton)
        deviceRowStackView.addArrangedSubview(spacerView)

        textsStackView.addArrangedSubview(deviceRowStackView)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: deviceRowStackView)
        textsStackView.addArrangedSubview(descriptionLabel)

        buttonsStackView.addArrangedSubview(purchaseButton)

        addSubview(textsStackView)
        addSubview(buttonsStackView)
        addConstraints()

        showCheckmark(false)
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
        [purchaseButton, infoButton, copyButton].forEach {
            $0.addTarget(self, action: #selector(tapped(button:)), for: .touchUpInside)
        }
    }

    @objc private func tapped(button: AppButton) {
        switch button.accessibilityIdentifier {
        case AccessibilityIdentifier.purchaseButton.asString:
            delegate?.didTapPurchaseButton(welcomeContentView: self, button: button)
        case AccessibilityIdentifier.infoButton.asString:
            delegate?.didTapInfoButton(welcomeContentView: self, button: button)
        case AccessibilityIdentifier.copyButton.asString:
            didTapCopyAccountNumber()
        default: return
        }
    }

    private func showCheckmark(_ showCheckmark: Bool) {
        if showCheckmark {
            let tickIcon = UIImage(named: "IconTick")

            copyButton.setImage(tickIcon, for: .normal)
            copyButton.tintColor = .successColor
        } else {
            let copyIcon = UIImage(named: "IconCopy")

            copyButton.setImage(copyIcon, for: .normal)
            copyButton.tintColor = .white
        }
    }

    @objc private func didTapCopyAccountNumber() {
        let delayedWorkItem = DispatchWorkItem { [weak self] in
            self?.showCheckmark(false)
        }

        revertCopyImageWorkItem?.cancel()
        revertCopyImageWorkItem = delayedWorkItem

        showCheckmark(true)
        delegate?.didTapCopyButton(welcomeContentView: self, button: copyButton)

        DispatchQueue.main.asyncAfter(
            deadline: .now() + .seconds(2),
            execute: delayedWorkItem
        )
    }
}
