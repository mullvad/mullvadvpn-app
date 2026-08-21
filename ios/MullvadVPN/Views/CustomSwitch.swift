// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

class CustomSwitch: UISwitch {
    /// Returns the private `UISwitch` background view
    private var backgroundView: UIView? {
        // Go two levels deep only
        let subviewsToExamine = subviews.flatMap { view -> [UIView] in
            [view] + view.subviews
        }

        // Find the first subview that has background color set.
        let backgroundView = subviewsToExamine.first { subview in
            subview.backgroundColor != nil
        }

        return backgroundView
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        tintColor = .clear
        onTintColor = .clear
        overrideUserInterfaceStyle = .light

        setAccessibilityIdentifier(.customSwitch)

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
