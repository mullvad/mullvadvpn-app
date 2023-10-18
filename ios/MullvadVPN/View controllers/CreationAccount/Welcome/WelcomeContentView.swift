//
//  WelcomeContentView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol WelcomeContentViewDelegate: AnyObject {
    func didTapPurchaseButton(welcomeContentView: WelcomeContentView, button: AppButton)
    func didTapRedeemVoucherButton(welcomeContentView: WelcomeContentView, button: AppButton)
    func didTapInfoButton(welcomeContentView: WelcomeContentView, button: UIButton)
}

struct WelcomeViewModel {
    let deviceName: String
    let accountNumber: String
}

final class WelcomeContentView: UIView {
    private enum Action: String {
        case purchase, redeemVoucher, showInfo
    }

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
        label.adjustsFontForContentSizeCategory = true
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = .zero
        label.font = .preferredFont(forTextStyle: .title2, weight: .bold)
        label.textColor = .white
        return label
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
        button.accessibilityIdentifier = Action.showInfo.rawValue
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
        button.accessibilityIdentifier = Action.purchase.rawValue
        let localizedString = NSLocalizedString(
            "BUY_CREDIT_BUTTON",
            tableName: "Welcome",
            value: "Buy credit",
            comment: ""
        )
        button.setTitle(localizedString, for: .normal)
        return button
    }()

    private let redeemVoucherButton: AppButton = {
        let button = AppButton(style: .success)
        button.accessibilityIdentifier = Action.redeemVoucher.rawValue
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_BUTTON_TITLE",
            tableName: "Account",
            value: "Redeem voucher",
            comment: ""
        ), for: .normal)
        return button
    }()

    private let textsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.axis = .vertical
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

    var isPurchasing = false {
        didSet {
            let alpha = isPurchasing ? 0.7 : 1.0
            purchaseButton.isLoading = isPurchasing
            purchaseButton.alpha = alpha
            redeemVoucherButton.isEnabled = !isPurchasing
            redeemVoucherButton.alpha = alpha
        }
    }

    var productState: ProductState = .none {
        didSet {
            purchaseButton.setTitle(productState.purchaseButtonTitle, for: .normal)
            purchaseButton.isLoading = productState.isFetching
            purchaseButton.isEnabled = productState.isReceived
        }
    }

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
        textsStackView.addArrangedSubview(subtitleLabel)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: subtitleLabel)
        textsStackView.addArrangedSubview(accountNumberLabel)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: accountNumberLabel)

        deviceRowStackView.addArrangedSubview(deviceNameLabel)
        deviceRowStackView.setCustomSpacing(UIMetrics.padding8, after: deviceNameLabel)
        deviceRowStackView.addArrangedSubview(infoButton)
        deviceRowStackView.addArrangedSubview(spacerView)

        textsStackView.addArrangedSubview(deviceRowStackView)
        textsStackView.setCustomSpacing(UIMetrics.padding16, after: deviceRowStackView)
        textsStackView.addArrangedSubview(descriptionLabel)

        buttonsStackView.addArrangedSubview(purchaseButton)
        #if DEBUG
        buttonsStackView.addArrangedSubview(redeemVoucherButton)
        #endif

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
        [redeemVoucherButton, purchaseButton, infoButton].forEach {
            $0.addTarget(self, action: #selector(tapped(button:)), for: .touchUpInside)
        }
    }

    @objc private func tapped(button: AppButton) {
        switch button.accessibilityIdentifier {
        case Action.purchase.rawValue:
            delegate?.didTapPurchaseButton(welcomeContentView: self, button: button)
        case Action.redeemVoucher.rawValue:
            delegate?.didTapRedeemVoucherButton(welcomeContentView: self, button: button)
        case Action.showInfo.rawValue:
            delegate?.didTapInfoButton(welcomeContentView: self, button: button)
        default: return
        }
    }
}
