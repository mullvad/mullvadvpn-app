//
//  UIBarButtonItem+Blocks.swift
//  MullvadVPN
//
//  Created by pronebird on 19/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

private var actionHandlerAssociatedKey = 0

extension UIBarButtonItem {
    typealias ActionHandler = () -> Void

    /**
     Block handler assigned to bar button item.
     */
    var actionHandler: ActionHandler? {
        get {
            return objc_getAssociatedObject(self, &actionHandlerAssociatedKey) as? ActionHandler
        }
        set {
            objc_setAssociatedObject(self, &actionHandlerAssociatedKey, newValue, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)

            target = newValue == nil ? nil : self
            action = newValue == nil ? nil : #selector(handleAction)
        }
    }

    /**
     Initialize bar button item with block handler.
     */
    convenience init(systemItem: UIBarButtonItem.SystemItem, actionHandler: @escaping ActionHandler) {
        self.init(barButtonSystemItem: systemItem, target: nil, action: nil)

        self.actionHandler = actionHandler
    }

    @objc private func handleAction() {
        actionHandler?()
    }
}
