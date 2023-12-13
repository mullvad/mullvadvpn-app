//
//  EditAccessMethodViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol EditAccessMethodViewControllerDelegate: AnyObject {
    /// The view controller requests the delegate to present the proxy configuration view controller.
    /// - Parameter controller: the calling controller.
    func controllerShouldShowProxyConfiguration(_ controller: EditAccessMethodViewController)

    /// The view controller deleted the access method.
    ///
    /// The delegate should consider dismissing the view controller.
    ///
    /// - Parameter controller: the calling controller.
    func controllerDidDeleteAccessMethod(_ controller: EditAccessMethodViewController)

    /// The view controller saved changes to the access method.
    ///
    /// The delegate should consider dismissing the view controller.
    ///
    /// - Parameter controller: the calling controller.
    func controllerDidSaveAccessMethod(_ controller: EditAccessMethodViewController)
}
