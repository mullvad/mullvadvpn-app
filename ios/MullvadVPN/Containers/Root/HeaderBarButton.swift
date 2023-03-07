//
//  HeaderBarButton.swift
//  MullvadVPN
//
//  Created by pronebird on 15/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class HeaderBarButton: UIButton {
    override func point(inside point: CGPoint, with event: UIEvent?) -> Bool {
        return bounds.insetBy(dx: -10, dy: -10).contains(point)
    }
}
