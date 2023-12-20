//
//  LinkButton.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-12-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
            .underlineStyle: NSUnderlineStyle.single.rawValue,
        ]

        if let titleColor = state.customButtonTitleColor {
            attributes[.foregroundColor] = titleColor
        }

        return NSAttributedString(string: title, attributes: attributes)
    }
}
