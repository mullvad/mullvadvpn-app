//
//  MethodSettingsValidationErrorContentView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting the access method validation errors.
class MethodSettingsValidationErrorContentView: UIView, UIContentView {
    let contentView = UIStackView()

    var icon: UIImageView {
        let view = UIImageView(image: UIImage(resource: .iconAlert).withTintColor(.dangerColor))
        view.heightAnchor.constraint(equalToConstant: 14).isActive = true
        view.widthAnchor.constraint(equalTo: view.heightAnchor, multiplier: 1).isActive = true
        return view
    }

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? MethodSettingsValidationErrorContentConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: MethodSettingsValidationErrorContentConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is MethodSettingsValidationErrorContentConfiguration
    }

    init(configuration: MethodSettingsValidationErrorContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

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

    private func configureSubviews(previousConfiguration: MethodSettingsValidationErrorContentConfiguration? = nil) {
        guard actualConfiguration != previousConfiguration else { return }

        configureLayoutMargins()

        contentView.arrangedSubviews.forEach { view in
            view.removeFromSuperview()
        }

        actualConfiguration.fieldErrors.forEach { error in
            let label = UILabel()
            label.text = error.errorDescription
            label.numberOfLines = 0
            label.font = .systemFont(ofSize: 13)
            label.textColor = .white.withAlphaComponent(0.6)

            let stackView = UIStackView(arrangedSubviews: [icon, label])
            stackView.alignment = .top
            stackView.spacing = 6

            contentView.addArrangedSubview(stackView)
        }
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }
}
