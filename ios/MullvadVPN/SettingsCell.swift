//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsCell: BasicTableViewCell {

    private let preferredMargins = UIEdgeInsets(top: 16, left: 24, bottom: 16, right: 12)
    private var appDidBecomeActiveObserver: NSObjectProtocol?

    override func awakeFromNib() {
        super.awakeFromNib()

        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor

        contentView.layoutMargins = preferredMargins
        separatorInset = .zero

        enableDisclosureViewTintColorFix()
    }

    /// `UITableViewCell` resets the disclosure view image when the app goes in background
    /// This fix ensures that the image is tinted when the app becomes active again.
    private func enableDisclosureViewTintColorFix() {
        appDidBecomeActiveObserver = NotificationCenter.default.addObserver(
            forName: UIApplication.didBecomeActiveNotification,
            object: nil,
            queue: nil) { [weak self] (note) in
                self?.updateDisclosureViewTintColor()
        }

        updateDisclosureViewTintColor()
    }

    /// For some reason the `tintColor` is not applied to standard accessory views.
    /// Fix this by looking for the accessory button and changing the image rendering mode
    private func updateDisclosureViewTintColor() {
        for case let button as UIButton in subviews {
            if let image = button.backgroundImage(for: .normal)?.withRenderingMode(.alwaysTemplate) {
                button.setBackgroundImage(image, for: .normal)
            }
        }
    }
}
