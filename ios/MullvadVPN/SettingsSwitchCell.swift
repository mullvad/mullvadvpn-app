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

    // MARK: -  Accessibility

    override var accessibilityTraits: UIAccessibilityTraits {
        set {
            super.accessibilityTraits = newValue
        }
        get {
            // Use UISwitch traits to make the entire cell behave as "Switch button"
            return switchContainer.control.accessibilityTraits
        }
    }

    override var accessibilityLabel: String? {
        set {
            super.accessibilityLabel = newValue
        }
        get {
            return titleLabel.text
        }
    }

    override var accessibilityValue: String? {
        set {
            super.accessibilityValue = newValue
        }
        get {
            return self.switchContainer.control.accessibilityValue
        }
    }

    override var accessibilityFrame: CGRect {
        set {
            super.accessibilityFrame = newValue
        }
        get {
            return UIAccessibility.convertToScreenCoordinates(self.bounds, in: self)
        }
    }

    override var accessibilityPath: UIBezierPath? {
        set {
            super.accessibilityPath = newValue
        }
        get {
            return UIBezierPath(roundedRect: accessibilityFrame, cornerRadius: 4)
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
