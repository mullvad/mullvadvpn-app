//
//  ListCellContentView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting a primary, secondary (trailing) and tertiary (below primary) label.
class ListCellContentView: UIView, UIContentView, UITextFieldDelegate {
    private var textLabel = UILabel()
    private var secondaryTextLabel = UILabel()
    private var tertiaryTextLabel = UILabel()
    private var tickImageView = UIImageView()
    private var breadcrumbImageView = UIImageView()
    private var contentView = UIStackView()
    private var containerView = UIStackView()

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
        configureTickImageView()
        configureBreadcrumbImageView()
        configureLayoutMargins()
    }

    override func didMoveToSuperview() {
        super.didMoveToSuperview()
        DispatchQueue.main.async {
            self.updateAxisIfNeeded()
        }
    }

    private func updateAxisIfNeeded() {
        let newAxis: NSLayoutConstraint.Axis = contentView.isOverflowed ? .vertical : .horizontal
        guard newAxis != contentView.axis else { return }
        contentView.axis = newAxis
        invalidateIntrinsicContentSize()
    }

    private func configureTextLabel() {
        let textProperties = actualConfiguration.textProperties

        textLabel.font = textProperties.font
        textLabel.adjustsFontForContentSizeCategory = true
        textLabel.textColor =
            actualConfiguration.isEnabled
            ? textProperties.color
            : .primaryTextColor.withAlphaComponent(0.2)
        textLabel.numberOfLines = 0
        textLabel.text = actualConfiguration.text
        textLabel.isHidden = actualConfiguration.text == nil
    }

    private func configureSecondaryTextLabel() {
        let textProperties = actualConfiguration.secondaryTextProperties

        secondaryTextLabel.font = textProperties.font
        secondaryTextLabel.adjustsFontForContentSizeCategory = true
        secondaryTextLabel.textColor =
            actualConfiguration.isEnabled
            ? textProperties.color
            : .primaryTextColor.withAlphaComponent(0.2)
        secondaryTextLabel.numberOfLines = 0
        secondaryTextLabel.textAlignment = .right
        secondaryTextLabel.text = actualConfiguration.secondaryText
        secondaryTextLabel.isHidden = actualConfiguration.secondaryText == nil
    }

    private func configureTertiaryTextLabel() {
        let textProperties = actualConfiguration.tertiaryTextProperties

        tertiaryTextLabel.font = textProperties.font
        tertiaryTextLabel.adjustsFontForContentSizeCategory = true
        tertiaryTextLabel.textColor =
            actualConfiguration.isEnabled
            ? textProperties.color
            : .primaryTextColor.withAlphaComponent(0.2)
        tertiaryTextLabel.numberOfLines = 0

        tertiaryTextLabel.text = actualConfiguration.tertiaryText
        tertiaryTextLabel.isHidden = actualConfiguration.tertiaryText == nil
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }

    private func configureTickImageView() {
        let image = UIImage.tick
            .withTintColor(.Cell.disclosureIndicatorColor)
            .withAlpha(actualConfiguration.isEnabled ? 1 : 0.2)

        tickImageView.image = image
        tickImageView.contentMode = .center
        tickImageView.widthAnchor.constraint(equalToConstant: 24).isActive = true
        tickImageView.isHidden = !actualConfiguration.isSelected
    }

    private func configureBreadcrumbImageView() {
        breadcrumbImageView.image = .stateIssue
        breadcrumbImageView.contentMode = .scaleAspectFit
        breadcrumbImageView.widthAnchor.constraint(equalToConstant: 18).isActive = true
        breadcrumbImageView.isHidden = !actualConfiguration.showBreadcrumb
    }

    private func addSubviews() {
        let leadingTextContainer = UIStackView(arrangedSubviews: [textLabel, tertiaryTextLabel])
        leadingTextContainer.axis = .vertical

        secondaryTextLabel.setContentCompressionResistancePriority(.required, for: .horizontal)

        contentView.addArrangedSubview(leadingTextContainer)
        contentView.addArrangedSubview(secondaryTextLabel)

        containerView.spacing = 8
        containerView.addArrangedSubview(tickImageView)
        containerView.addArrangedSubview(contentView)
        containerView.addArrangedSubview(breadcrumbImageView)

        addConstrainedSubviews([containerView]) {
            containerView.pinEdgesToSuperviewMargins()
        }
    }
}
