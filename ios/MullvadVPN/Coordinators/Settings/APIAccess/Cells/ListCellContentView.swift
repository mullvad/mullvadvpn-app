//
//  ListCellContentView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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
                  actualConfiguration != newConfiguration else { return }

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

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 0))

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
        textLabel.textColor = textProperties.color

        textLabel.text = actualConfiguration.text
    }

    private func configureSecondaryTextLabel() {
        let textProperties = actualConfiguration.secondaryTextProperties

        secondaryTextLabel.font = textProperties.font
        secondaryTextLabel.textColor = textProperties.color

        secondaryTextLabel.text = actualConfiguration.secondaryText
    }

    private func configureTertiaryTextLabel() {
        let textProperties = actualConfiguration.tertiaryTextProperties

        tertiaryTextLabel.font = textProperties.font
        tertiaryTextLabel.textColor = textProperties.color

        tertiaryTextLabel.text = actualConfiguration.tertiaryText
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }

    private func addSubviews() {
        let leadingTextContainer = UIStackView(arrangedSubviews: [textLabel, tertiaryTextLabel])
        leadingTextContainer.axis = .vertical

        addConstrainedSubviews([leadingTextContainer, secondaryTextLabel]) {
            leadingTextContainer.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
            leadingTextContainer.centerYAnchor.constraint(equalTo: centerYAnchor)
            secondaryTextLabel.pinEdgesToSuperviewMargins(.all().excluding(.leading))
            secondaryTextLabel.leadingAnchor.constraint(
                greaterThanOrEqualToSystemSpacingAfter: leadingTextContainer.trailingAnchor,
                multiplier: 1
            )
        }
    }
}
