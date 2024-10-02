//
//  EditAccessMethodViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol EditAccessMethodViewControllerDelegate: AnyObject, AccessMethodEditing {
    /// The view controller requests the delegate to present the proxy configuration view controller.
    /// - Parameter controller: the calling controller.
    func controllerShouldShowMethodSettings(_ controller: EditAccessMethodViewController)

    /// The view controller deleted the access method.
    ///
    /// The delegate should consider dismissing the view controller.
    ///
    /// - Parameter controller: the calling controller.
    func controllerDidDeleteAccessMethod(_ controller: EditAccessMethodViewController)

    /// The view controller requests the delegate to present information about the access method.
    /// - Parameter controller: the calling controller.
    func controllerShouldShowMethodInfo(_ controller: EditAccessMethodViewController, config: InfoModalConfig)
}
