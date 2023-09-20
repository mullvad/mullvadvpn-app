//
//  UIEdgeInsets+Extensions.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-09-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

extension UIEdgeInsets {
    var toDirectionalInsets: NSDirectionalEdgeInsets {
        NSDirectionalEdgeInsets(
            top: top,
            leading: left,
            bottom: bottom,
            trailing: right
        )
    }
}
