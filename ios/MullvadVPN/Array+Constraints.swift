//
//  Array+Constraints.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-09-02.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

extension Array where Element == NSLayoutConstraint {
    var height: NSLayoutConstraint? {
        first(where: { $0.firstAttribute == .height })
    }
}
