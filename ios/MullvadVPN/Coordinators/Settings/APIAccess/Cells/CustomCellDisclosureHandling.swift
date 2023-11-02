//
//  CustomCellDisclosureHandling.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Types handling custom disclosure accessory in table view cells.
protocol CustomCellDisclosureHandling: UITableViewCell {
    /// Custom disclosure type.
    ///
    /// Cannot be used together with `accessoryType` property. Automatically resets `accessoryType` upon assignment.
    var disclosureType: SettingsDisclosureType { get set }
}
