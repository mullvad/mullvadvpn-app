//
//  AddAccessMethodInteractorProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// The type implementing the interface for persisting the underlying access method view model in the new entry context.
protocol AddAccessMethodInteractorProtocol: ProxyConfigurationInteractorProtocol {
    /// Add new access method to the persistent store.
    ///
    /// - Calling this method multiple times does nothing as the entry with the same identifier cannot be added more than once.
    /// - View controllers should only call this method for valid view models, as this method will do nothing if the view model fails validation.
    func addMethod()
}
