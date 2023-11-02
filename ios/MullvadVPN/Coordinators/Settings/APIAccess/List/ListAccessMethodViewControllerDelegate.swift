//
//  ListAccessMethodViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol ListAccessMethodViewControllerDelegate: AnyObject {
    /// The view controller requests the delegate to present the about view.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerShouldShowAbout(_ controller: ListAccessMethodViewController)

    /// The view controller requests the delegate to present the add new method controller.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerShouldAddNew(_ controller: ListAccessMethodViewController)

    /// The view controller requests the delegate to present the view controller for editing the existing access method.
    ///
    /// - Parameters:
    ///   - controller: the calling view controller
    ///   - item: the selected item.
    func controller(_ controller: ListAccessMethodViewController, shouldEditItem item: ListAccessMethodItem)
}
