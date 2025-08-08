//
//  AccountExpiryRow.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class AccountExpiryRow: UIView {
    var value: Date? {
        didSet {
            let expiry = value

            if let expiry, expiry <= Date() {
                let localizedString = NSLocalizedString("OUT OF TIME", comment: "")

                valueLabel.text = localizedString
                accessibilityValue = localizedString

                valueLabel.textColor = .dangerColor
            } else {
                let formattedDate = expiry.map { date in
                    DateFormatter.localizedString(
                        from: date,
                        dateStyle: .medium,
                        timeStyle: .short
                    )
                }

                valueLabel.text = formattedDate ?? ""
                accessibilityValue = formattedDate

                valueLabel.textColor = .white
            }
        }
    }

    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.text = NSLocalizedString("Paid until", comment: "")
        textLabel.font = .mullvadTiny
        textLabel.numberOfLines = 0
        textLabel.adjustsFontForContentSizeCategory = true
        textLabel.textColor = UIColor(white: 1.0, alpha: 0.6)
        return textLabel
    }()

    private let valueLabel: UILabel = {
        let valueLabel = UILabel()
        valueLabel.translatesAutoresizingMaskIntoConstraints = false
        valueLabel.font = .mullvadSmall
        valueLabel.adjustsFontForContentSizeCategory = true
        valueLabel.textColor = .white
        valueLabel.numberOfLines = 0
        valueLabel.setAccessibilityIdentifier(.accountPagePaidUntilLabel)
        return valueLabel
    }()

    let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .small)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        activityIndicator.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        activityIndicator.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return activityIndicator
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addConstrainedSubviews([textLabel, activityIndicator, valueLabel]) {
            textLabel.pinEdgesToSuperview(.all().excluding([.trailing, .bottom]))
            textLabel.trailingAnchor.constraint(
                greaterThanOrEqualTo: activityIndicator.leadingAnchor,
                constant: -UIMetrics.padding8
            )

            activityIndicator.topAnchor.constraint(equalTo: textLabel.topAnchor)
            activityIndicator.bottomAnchor.constraint(equalTo: textLabel.bottomAnchor)
            activityIndicator.trailingAnchor.constraint(equalTo: trailingAnchor)

            valueLabel.pinEdgesToSuperview(.all().excluding(.top))
            valueLabel.topAnchor.constraint(equalTo: textLabel.bottomAnchor, constant: UIMetrics.padding8)
        }
        isAccessibilityElement = true
        accessibilityLabel = textLabel.text
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
