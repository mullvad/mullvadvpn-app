//
//  AddAccessMethodViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol AddAccessMethodViewControllerDelegate: AnyObject {
    /// The view controller added the API access method.
    ///
    /// The delegate should consider dismissing the view controller.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerDidAdd(_ controller: AddAccessMethodViewController)

    /// The user cancelled the view controller.
    ///
    /// The delegate should consider dismissing the view controller.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerDidCancel(_ controller: AddAccessMethodViewController)

    /// The view controller requests the delegate to present the API access method protocol picker.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerShouldShowProtocolPicker(_ controller: AddAccessMethodViewController)

    /// The view controller requests the delegate to present the cipher picker.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerShouldShowShadowsocksCipherPicker(_ controller: AddAccessMethodViewController)
}
