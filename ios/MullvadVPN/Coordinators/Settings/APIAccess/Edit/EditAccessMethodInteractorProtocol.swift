//
//  EditAccessMethodInteractorProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

/// The type implementing the interface for persisting changes to the underlying access method view model in the editing context.
protocol EditAccessMethodInteractorProtocol: ProxyConfigurationInteractorProtocol, AccessMethodRepositoryDataSource {
    /// Save changes to persistent store.
    ///
    /// - Calling this method when the underlying view model fails validation does nothing.
    /// - View controllers are responsible to validate the view model before calling this method.
    func saveAccessMethod()

    /// Delete the access method from persistent store.
    ///
    /// - Calling this method multiple times does nothing.
    /// - View model does not have to pass validation for this method to work as the identifier field is the only requirement.
    /// - View controller presenting the UI for editing the access method must be dismissed after calling this method.
    func deleteAccessMethod()
}
