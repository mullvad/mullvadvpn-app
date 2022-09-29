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
    private static func makeSpacerView() -> UIView {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.setContentCompressionResistancePriority(.defaultLow, for: .vertical)
        view.setContentHuggingPriority(.defaultLow, for: .vertical)
        return view
    }

    private let topSpacerView = makeSpacerView()
    private let bottomSpacerView = makeSpacerView()

    let statusActivityView: StatusActivityView = {
        let statusActivityView = StatusActivityView(state: .failure)
        statusActivityView.translatesAutoresizingMaskIntoConstraints = false
        return statusActivityView
    }()

    let titleLabel: UILabel = {
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

    let bodyLabel: UILabel = {
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

    let disconnectButton: AppButton = {
        let button = AppButton(style: .danger)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.alpha = 0
        let localizedString = NSLocalizedString(
            "OUT_OF_TIME_DISCONNECT_BUTTON",
            tableName: "OutOfTime",
            value: "Disconnect",
            comment: ""
        )
        button.setTitle(localizedString, for: .normal)
        return button
    }()

    let restoreButton: AppButton = {
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

    let purchaseButton: InAppPurchaseButton = {
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

    let redeemButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_BUTTON_TITLE",
            tableName: "OutOfTime",
            value: "Redeem voucher",
            comment: ""
        ), for: .normal)
        return button
    }()

    private lazy var topStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [statusActivityView, titleLabel, bodyLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.StackSpacing.regular
        return stackView
    }()

    private lazy var bottomStackView: UIStackView = {
        let stackView = UIStackView(
            arrangedSubviews: [disconnectButton, purchaseButton, redeemButton, restoreButton]
        )
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.StackSpacing.regular
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        layoutMargins = UIMetrics.contentLayoutMargins

        addSubviews()
        addConstraints()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        for subview in [topSpacerView, topStackView, bottomSpacerView, bottomStackView] {
            addSubview(subview)
        }
    }

    private func addConstraints() {
        NSLayoutConstraint.activate([
            topSpacerView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            topSpacerView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            topSpacerView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            topSpacerView.heightAnchor.constraint(equalTo: bottomSpacerView.heightAnchor),

            topStackView.topAnchor.constraint(equalTo: topSpacerView.bottomAnchor),
            topStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),
            topStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),

            bottomSpacerView.topAnchor.constraint(equalTo: topStackView.bottomAnchor),
            bottomSpacerView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            bottomSpacerView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            bottomSpacerView.heightAnchor
                .constraint(greaterThanOrEqualToConstant: UIMetrics.sectionSpacing),

            bottomStackView.topAnchor.constraint(equalTo: bottomSpacerView.bottomAnchor),
            bottomStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),
            bottomStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),
            bottomStackView.bottomAnchor.constraint(
                equalTo: layoutMarginsGuide.bottomAnchor
            ),
        ])
    }
}
