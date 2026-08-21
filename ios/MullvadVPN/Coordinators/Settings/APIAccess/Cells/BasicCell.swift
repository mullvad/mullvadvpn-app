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

/// Basic cell that supports dynamic background configuration and custom cell disclosure.
class BasicCell: UITableViewCell, DynamicBackgroundConfiguration, CustomCellDisclosureHandling {
    private lazy var disclosureImageView = UIImageView(image: nil)

    var backgroundConfigurationResolver: BackgroundConfigurationResolver? {
        didSet {
            backgroundConfiguration = backgroundConfigurationResolver?(configurationState)
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func updateConfiguration(using state: UICellConfigurationState) {
        if let backgroundConfiguration = backgroundConfigurationResolver?(state) {
            self.backgroundConfiguration = backgroundConfiguration
        } else {
            super.updateConfiguration(using: state)
        }
    }

    var disclosureType: SettingsDisclosureType = .none {
        didSet {
            accessoryType = .none

            guard
                let image = disclosureType.image?.withTintColor(
                    UIColor.Cell.disclosureIndicatorColor,
                    renderingMode: .alwaysOriginal
                )
            else {
                accessoryView = nil
                return
            }

            disclosureImageView.image = image
            disclosureImageView.sizeToFit()
            accessoryView = disclosureImageView
        }
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        disclosureType = .none
    }
}
