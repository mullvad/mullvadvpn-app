//
//  SettingsSwitchCell.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsSwitchCell: SettingsCell {
    private let switchContainer = CustomSwitchContainer()

    var action: ((Bool) -> Void)?

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        accessoryView = switchContainer

        switchContainer.control.addTarget(
            self,
            action: #selector(switchValueDidChange),
            for: .valueChanged
        )

        isAccessibilityElement = true
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setEnabled(_ isEnabled: Bool) {
        switchContainer.isEnabled = isEnabled
    }

    func setOn(_ isOn: Bool, animated: Bool) {
        switchContainer.control.setOn(isOn, animated: animated)
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setEnabled(true)
    }

    // MARK: - Actions

    @objc private func switchValueDidChange() {
        action?(switchContainer.control.isOn)
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
            titleLabel.text
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

        setOn(newValue, animated: true)
        action?(newValue)

        return true
    }
}
