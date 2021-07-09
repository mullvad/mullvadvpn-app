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

        switchContainer.control.addTarget(self, action: #selector(switchValueDidChange), for: .valueChanged)

        // Use UISwitch traits to make the entire cell behave as "Switch button"
        accessibilityTraits = switchContainer.control.accessibilityTraits
        isAccessibilityElement = true
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setOn(_ isOn: Bool, animated: Bool) {
        switchContainer.control.setOn(isOn, animated: animated)
    }

    // MARK: - Actions

    @objc private func switchValueDidChange() {
        action?(self.switchContainer.control.isOn)
    }

    // MARK: -  Accessibility

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
        let newValue = !self.switchContainer.control.isOn

        setOn(newValue, animated: true)
        action?(newValue)

        return true
    }
}
