//
//  CustomSwitch.swift
//  MullvadVPN
//
//  Created by pronebird on 20/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomSwitch: UISwitch {
    /// Returns the private `UISwitch` background view
    private var backgroundView: UIView? {
        // Go two levels deep only
        let subviewsToExamine = subviews.flatMap { view -> [UIView] in
            return [view] + view.subviews
        }

        // Find the first subview that has background color set.
        let backgroundView = subviewsToExamine.first { subview in
            return subview.backgroundColor != nil
        }

        return backgroundView
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        tintColor = .clear
        onTintColor = .clear
        overrideUserInterfaceStyle = .light

        updateThumbColor(isOn: isOn, animated: false)

        addTarget(self, action: #selector(valueChanged(_:)), for: .valueChanged)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func setOn(_ on: Bool, animated: Bool) {
        super.setOn(on, animated: animated)

        updateThumbColor(isOn: on, animated: animated)
    }

    private func updateThumbColor(isOn: Bool, animated: Bool) {
        let actions = {
            self.thumbTintColor = isOn ? UIColor.Switch.onThumbColor : UIColor.Switch.offThumbColor
            self.backgroundView?.backgroundColor = .clear
        }

        if animated {
            UIView.animate(withDuration: 0.25, animations: actions)
        } else {
            actions()
        }
    }

    @objc private func valueChanged(_ sender: Any) {
        updateThumbColor(isOn: isOn, animated: true)
    }
}
