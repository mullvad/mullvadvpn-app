//
//  NotificationContainerView.swift
//  MullvadVPN
//
//  Created by pronebird on 03/06/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class NotificationContainerView: UIView {
    override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
        let hitView = super.hitTest(point, with: event)

        // Pass through taps if no subview was touched
        if hitView == self {
            return nil
        } else {
            return hitView
        }
    }
}
