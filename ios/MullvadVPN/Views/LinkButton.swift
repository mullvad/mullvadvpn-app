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

/// A subclass that implements the button that visually look like URL links on the web
class LinkButton: CustomButton {
    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    var titleString: String? {
        didSet {
            updateAttributedTitle(string: titleString)
        }
    }

    private func commonInit() {
        imageAlignment = .trailing
        contentHorizontalAlignment = .leading
        configuration?.contentInsets = .zero
        accessibilityTraits.insert(.link)
    }

    private func updateAttributedTitle(string: String?) {
        let states: [UIControl.State] = [.normal, .highlighted, .disabled]
        states.forEach { state in
            let attributedTitle = string.flatMap { makeAttributedTitle($0, for: state) }
            self.setAttributedTitle(attributedTitle, for: state)
        }
    }

    private func makeAttributedTitle(
        _ title: String,
        for state: UIControl.State
    ) -> NSAttributedString {
        var attributes: [NSAttributedString.Key: Any] = [
            .underlineStyle: NSUnderlineStyle.single.rawValue
        ]

        if let titleColor = state.customButtonTitleColor {
            attributes[.foregroundColor] = titleColor
        }

        return NSAttributedString(string: title, attributes: attributes)
    }
}
