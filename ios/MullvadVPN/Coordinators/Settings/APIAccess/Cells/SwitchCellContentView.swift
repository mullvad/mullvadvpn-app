//
//  SwitchCellContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content view presenting a label and switch control.
class SwitchCellContentView: UIView, UIContentView, UITextFieldDelegate {
    private var textLabel = UILabel()
    private let switchContainer = CustomSwitchContainer()

    var configuration: UIContentConfiguration {
        get {
            actualConfiguration
        }
        set {
            guard let newConfiguration = newValue as? SwitchCellContentConfiguration,
                  actualConfiguration != newConfiguration else { return }

            let previousConfiguration = actualConfiguration
            actualConfiguration = newConfiguration

            configureSubviews(previousConfiguration: previousConfiguration)
        }
    }

    private var actualConfiguration: SwitchCellContentConfiguration

    func supports(_ configuration: UIContentConfiguration) -> Bool {
        configuration is SwitchCellContentConfiguration
    }

    init(configuration: SwitchCellContentConfiguration) {
        actualConfiguration = configuration

        super.init(frame: CGRect(x: 0, y: 0, width: 100, height: 44))

        configureSubviews()
        addSubviews()
        configureAccessibility()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configureSubviews(previousConfiguration: SwitchCellContentConfiguration? = nil) {
        configureTextLabel()
        configureSwitch()
        configureLayoutMargins()
        configureActions(previousConfiguration: previousConfiguration)
    }

    private func configureActions(previousConfiguration: SwitchCellContentConfiguration? = nil) {
        previousConfiguration?.onChange.map { switchContainer.control.removeAction($0, for: .valueChanged) }
        actualConfiguration.onChange.map { switchContainer.control.addAction($0, for: .valueChanged) }
    }

    private func configureLayoutMargins() {
        directionalLayoutMargins = actualConfiguration.directionalLayoutMargins
    }

    private func configureTextLabel() {
        let textProperties = actualConfiguration.textProperties

        textLabel.font = textProperties.font
        textLabel.textColor = textProperties.color

        textLabel.text = actualConfiguration.text
    }

    private func configureSwitch() {
        switchContainer.control.isOn = actualConfiguration.isOn
    }

    private func addSubviews() {
        addConstrainedSubviews([textLabel, switchContainer]) {
            textLabel.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
            switchContainer.centerYAnchor.constraint(equalTo: centerYAnchor)
            switchContainer.pinEdgeToSuperviewMargin(.trailing(0))
            switchContainer.leadingAnchor.constraint(
                greaterThanOrEqualToSystemSpacingAfter: textLabel.trailingAnchor,
                multiplier: 1
            )
        }
    }

    private func configureAccessibility() {
        isAccessibilityElement = true
    }

    // MARK: - Accessibility

    override var accessibilityTraits: UIAccessibilityTraits {
        get {
            // Use UISwitch traits to make the entire cell behave as "Switch button"
            switchContainer.control.accessibilityTraits
        }
        set {
            super.accessibilityTraits = newValue
        }
    }

    override var accessibilityLabel: String? {
        get {
            actualConfiguration.text
        }
        set {
            super.accessibilityLabel = newValue
        }
    }

    override var accessibilityValue: String? {
        get {
            self.switchContainer.control.accessibilityValue
        }
        set {
            super.accessibilityValue = newValue
        }
    }

    override var accessibilityFrame: CGRect {
        get {
            UIAccessibility.convertToScreenCoordinates(self.bounds, in: self)
        }
        set {
            super.accessibilityFrame = newValue
        }
    }

    override var accessibilityPath: UIBezierPath? {
        get {
            UIBezierPath(roundedRect: accessibilityFrame, cornerRadius: 4)
        }
        set {
            super.accessibilityPath = newValue
        }
    }

    override func accessibilityActivate() -> Bool {
        guard switchContainer.isEnabled else { return false }

        let newValue = !switchContainer.control.isOn

        switchContainer.control.setOn(newValue, animated: true)
        switchContainer.control.sendActions(for: .valueChanged)

        return true
    }
}
