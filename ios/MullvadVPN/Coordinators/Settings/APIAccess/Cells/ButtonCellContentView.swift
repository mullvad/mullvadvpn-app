//
//  ButtonCellContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting a full-width button.
class ButtonCellContentView: UIView, UIContentView {
    private let button = AppButton()

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? ButtonCellContentConfiguration,
                  actualConfiguration != newConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: ButtonCellContentConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is ButtonCellContentConfiguration
    }

    init(configuration: ButtonCellContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

        configureSubviews()
        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configureSubviews(previousConfiguration: ButtonCellContentConfiguration? = nil) {
        guard actualConfiguration != previousConfiguration else { return }

        configureButton()
        configureActions(previousConfiguration: previousConfiguration)
    }

    private func configureActions(previousConfiguration: ButtonCellContentConfiguration? = nil) {
        previousConfiguration?.primaryAction.map { button.removeAction($0, for: .touchUpInside) }
        actualConfiguration.primaryAction.map { button.addAction($0, for: .touchUpInside) }
    }

    private func configureButton() {
        button.setTitle(actualConfiguration.text, for: .normal)
        button.isEnabled = actualConfiguration.isEnabled
        button.style = actualConfiguration.style
        button.overrideContentEdgeInsets = true
        button.directionalContentEdgeInsets = actualConfiguration.directionalContentEdgeInsets
    }

    private func addSubviews() {
        addConstrainedSubviews([button]) {
            button.pinEdgesToSuperview()
        }
    }
}
