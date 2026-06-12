//
//  SettingsFieldValidationErrorContentView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsFieldValidationErrorContentView: UIView, UIContentView {
    let contentView = UIStackView()

    var icon: DynamicImageView {
        let imageView = DynamicImageView(
            image: UIImage.Buttons.alert.withTintColor(.dangerColor), baseSize: 18, textStyle: .subheadline)
        imageView.contentMode = .scaleAspectFit
        return imageView
    }

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? SettingsFieldValidationErrorConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: SettingsFieldValidationErrorConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is SettingsFieldValidationErrorConfiguration
    }

    init(configuration: SettingsFieldValidationErrorConfiguration) {
        actualConfiguration = configuration

        super.init(frame: .zero)

        addSubviews()
        configureSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        contentView.axis = .vertical
        contentView.spacing = 6

        addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperviewMargins()
        }
    }

    private func configureSubviews(previousConfiguration: SettingsFieldValidationErrorConfiguration? = nil) {
        guard actualConfiguration != previousConfiguration else { return }

        configureLayoutMargins()

        contentView.arrangedSubviews.forEach { view in
            view.removeFromSuperview()
        }

        actualConfiguration.errors.forEach { error in
            let label = UILabel()
            label.text = error.errorDescription
            label.numberOfLines = 0
            label.adjustsFontForContentSizeCategory = true
            label.font = .mullvadTiny
            label.setContentHuggingPriority(.defaultLow, for: .horizontal)  // Allow growing
            label.setContentCompressionResistancePriority(.required, for: .horizontal)
            label.textColor = .white

            icon.setContentHuggingPriority(.defaultHigh, for: .horizontal)  // Resist growing
            icon.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

            let stackView = UIStackView(arrangedSubviews: [icon, label])
            stackView.alignment = .center
            stackView.spacing = 4

            contentView.addArrangedSubview(stackView)
        }
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }
}
