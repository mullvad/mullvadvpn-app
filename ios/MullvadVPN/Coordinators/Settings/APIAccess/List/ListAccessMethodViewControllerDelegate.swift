// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

protocol ListAccessMethodViewControllerDelegate: AnyObject {
    /// The view controller requests the delegate to present the about view.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerShouldShowAbout()

    /// The view controller requests the delegate to present the add new method controller.
    ///
    /// - Parameter controller: the calling view controller.
    func controllerShouldAddNew()

    /// The view controller requests the delegate to present the view controller for editing the existing access method.
    ///
    /// - Parameters:
    ///   - controller: the calling view controller
    ///   - item: the selected item.
    func controller(shouldEditItem item: ListAccessMethodItem)
}
