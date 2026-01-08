//
//  ListCellContentView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting a primary, secondary (trailing) and tertiary (below primary) label.
class ListCellContentView: UIView, UIContentView, UITextFieldDelegate {
    private var textLabel = UILabel()
    private var secondaryTextLabel = UILabel()
    private var tertiaryTextLabel = UILabel()

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? ListCellContentConfiguration,
                actualConfiguration != newConfiguration
            else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: ListCellContentConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is ListCellContentConfiguration
    }

    init(configuration: ListCellContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: .zero)

        configureSubviews()
        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configureSubviews(previousConfiguration: ListCellContentConfiguration? = nil) {
        configureTextLabel()
        configureSecondaryTextLabel()
        configureTertiaryTextLabel()
        configureLayoutMargins()
    }

    private func configureTextLabel() {
        let textProperties = actualConfiguration.textProperties

        textLabel.font = textProperties.font
        textLabel.adjustsFontForContentSizeCategory = true
        textLabel.textColor = textProperties.color
        textLabel.numberOfLines = 0
        textLabel.text = actualConfiguration.text
        textLabel.setContentHuggingPriority(.defaultLow, for: .horizontal)
        textLabel.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
    }

    private func configureSecondaryTextLabel() {
        let textProperties = actualConfiguration.secondaryTextProperties

        secondaryTextLabel.font = textProperties.font
        secondaryTextLabel.adjustsFontForContentSizeCategory = true
        secondaryTextLabel.textColor = textProperties.color
        secondaryTextLabel.numberOfLines = 0
        secondaryTextLabel.text = actualConfiguration.secondaryText
    }

    private func configureTertiaryTextLabel() {
        let textProperties = actualConfiguration.tertiaryTextProperties

        tertiaryTextLabel.font = textProperties.font
        tertiaryTextLabel.adjustsFontForContentSizeCategory = true
        tertiaryTextLabel.textColor = textProperties.color
        tertiaryTextLabel.numberOfLines = 0

        tertiaryTextLabel.text = actualConfiguration.tertiaryText
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }

    private func addSubviews() {
        let leadingTextContainer = UIStackView(arrangedSubviews: [textLabel, tertiaryTextLabel])
        leadingTextContainer.axis = .vertical

        leadingTextContainer.setContentHuggingPriority(.defaultLow, for: .horizontal)
        leadingTextContainer.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)

        secondaryTextLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        secondaryTextLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        addConstrainedSubviews([leadingTextContainer, secondaryTextLabel]) {
            if actualConfiguration.secondaryText == nil {
                leadingTextContainer.pinEdgesToSuperviewMargins(.all())
            } else {
                leadingTextContainer.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
                secondaryTextLabel.pinEdgesToSuperviewMargins(.all().excluding(.leading))
                secondaryTextLabel.leadingAnchor.constraint(
                    greaterThanOrEqualToSystemSpacingAfter: leadingTextContainer.trailingAnchor,
                    multiplier: 1
                )
            }
        }
    }
}
