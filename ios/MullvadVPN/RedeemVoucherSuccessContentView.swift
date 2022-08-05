//
//  RedeemVoucherSuccessContentView.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-24.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class RedeemVoucherSuccessContentView: UIView {
    let timeAdded: String
    let newExpiry: String

    let successImage: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "IconSuccess"))
        imageView.translatesAutoresizingMaskIntoConstraints = false
        imageView.contentMode = .scaleAspectFit
        return imageView
    }()

    private lazy var titleLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_SUCCESS_TITLE",
            tableName: "RedeemVoucherSuccess",
            value: "Voucher was successfully redeemed",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 32)
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    lazy var bodyLabel: UILabel = {
        let label = UILabel()
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_SUCCESS_BODY",
            tableName: "RedeemVoucherSuccess",
            value: "\(timeAdded) was added, account paid until \(newExpiry).",
            comment: ""
        )
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        label.numberOfLines = 0
        return label
    }()

    private lazy var topStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [successImage, titleLabel, bodyLabel])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = UIMetrics.sectionSpacing
        return stackView
    }()

    let nextButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_SUCCESS_NEXT_BUTTON",
            tableName: "RedeemVoucherSuccess",
            value: "Next",
            comment: ""
        ), for: .normal)
        return button
    }()

    init(timeAdded: String, paidUntil: String) {
        self.timeAdded = timeAdded
        newExpiry = paidUntil
        super.init(frame: .zero)

        translatesAutoresizingMaskIntoConstraints = false

        backgroundColor = .secondaryColor

        setUpSubviews()

        layoutMargins = UIMetrics.contentLayoutMargins
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

private extension RedeemVoucherSuccessContentView {
    func setUpSubviews() {
        addSubview(topStackView)
        addSubview(nextButton)
        configureConstraints()
    }

    func configureConstraints() {
        NSLayoutConstraint.activate([
            topStackView.centerYAnchor.constraint(
                equalTo: centerYAnchor,
                constant: UIMetrics.verticalCenterOffset
            ),

            topStackView.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),

            topStackView.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),

            nextButton.leadingAnchor.constraint(
                equalTo: layoutMarginsGuide.leadingAnchor
            ),

            nextButton.trailingAnchor.constraint(
                equalTo: layoutMarginsGuide.trailingAnchor
            ),

            nextButton.bottomAnchor.constraint(
                equalTo: layoutMarginsGuide.bottomAnchor
            ),
        ])
    }
}
