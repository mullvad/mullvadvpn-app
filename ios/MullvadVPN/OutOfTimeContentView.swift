//
//  OutOfTimeContentView.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-07-26.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class OutOfTimeContentView: UIView {
    private lazy var failImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "IconFail"))
        imageView.contentMode = .scaleAspectFit
        return imageView
    }()

    private lazy var titleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "OUT_OF_TIME_TITLE",
            tableName: "OutOfTime",
            value: "Out of time",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 32)
        label.textColor = .white
        return label
    }()

    private lazy var bodyLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "OUT_OF_TIME_BODY",
            tableName: "OutOfTime",
            value: "You have no more VPN time left on this account. Either buy credit on our website or redeem a voucher.",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    lazy var purchaseButton: InAppPurchaseButton = {
        let button = InAppPurchaseButton()
        button.translatesAutoresizingMaskIntoConstraints = false
        let localizedString = NSLocalizedString(
            "OUT_OF_TIME_PURCHASE_BUTTON",
            tableName: "OutOfTime",
            value: "Add 30 days time",
            comment: ""
        )
        button.setTitle(localizedString, for: .normal)
        return button
    }()

    // TODO: Implement properly
    lazy var redeemButton: AppButton = {
        let button = AppButton(style: .success)
        button.setTitle(NSLocalizedString(
            "OUT_OF_TIME_REDEEM_BUTTON",
            tableName: "OutOfTime",
            value: "Redeem voucher",
            comment: ""
        ), for: .normal)
        return button
    }()

    lazy var restoreButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "RESTORE_PURCHASES_BUTTON_TITLE",
            tableName: "OutOfTime",
            value: "Restore purchases",
            comment: ""
        ), for: .normal)
        return button
    }()

    private lazy var topStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [failImageView, titleLabel, bodyLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    private lazy var bottomStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [purchaseButton, redeemButton, restoreButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        disableAutoresizingMask()
        setBackgroundColor()
        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

// MARK: - Private Functions

private extension OutOfTimeContentView {
    func disableAutoresizingMask() {
        translatesAutoresizingMaskIntoConstraints = false
    }

    func setBackgroundColor() {
        backgroundColor = .secondaryColor
    }

    func addSubviews() {
        addSubview(topStackView)
        addSubview(bottomStackView)
        configureConstraints()
    }

    func configureConstraints() {
        NSLayoutConstraint.activate([
            topStackView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor, constant: 24),
            topStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor,
                constant: 16
            ),
            topStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor,
                constant: -16
            ),

            purchaseButton.heightAnchor.constraint(equalToConstant: 48),
            redeemButton.heightAnchor.constraint(equalTo: purchaseButton.heightAnchor),
            restoreButton.heightAnchor.constraint(equalTo: purchaseButton.heightAnchor),

            bottomStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor,
                constant: 16
            ),
            bottomStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor,
                constant: -16
            ),
            bottomStackView.bottomAnchor.constraint(
                equalTo: layoutMarginsGuide.bottomAnchor,
                constant: -16
            ),
        ])
    }
}
