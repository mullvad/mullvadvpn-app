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
    let statusActivityView: StatusActivityView = {
        let statusActivityView = StatusActivityView(state: .failure)
        statusActivityView.translatesAutoresizingMaskIntoConstraints = false
        return statusActivityView
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
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    lazy var disconnectButton: AppButton = {
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
        let stackView = UIStackView(arrangedSubviews: [statusActivityView, titleLabel, bodyLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    private lazy var bottomStackView: UIStackView = {
        let stackView = UIStackView(
            arrangedSubviews: [disconnectButton, purchaseButton, restoreButton]
        )
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)
        translatesAutoresizingMaskIntoConstraints = false
        backgroundColor = .secondaryColor
        layoutMargins = UIMetrics.contentLayoutMargins
        setUpSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Private Functions

    func setUpSubviews() {
        addSubview(topStackView)
        addSubview(bottomStackView)
        configureConstraints()
    }

    func configureConstraints() {
        NSLayoutConstraint.activate([
            topStackView.centerYAnchor.constraint(equalTo: centerYAnchor, constant: -20),

            topStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),
            topStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),

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

    func setBodyLabelText(_ text: String) {
        bodyLabel.attributedText = NSAttributedString(
            markdownString: text,
            font: UIFont.systemFont(ofSize: 17)
        )
    }
}
