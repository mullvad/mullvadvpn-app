//
//  ProxyConfigurationInteractorProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// The type implementing the facilities for testing proxy configuration.
protocol ProxyConfigurationInteractorProtocol {
    /// Start testing proxy configuration with data from view model.
    ///
    /// - It's expected that the completion handler is not called if testing is cancelled.
    /// - The interactor should update the underlying view model to indicate the progress of testing. The view controller is expected to keep track of that and update
    ///   the UI accordingly.
    ///
    /// - Parameter completion: completion handler receiving `true` if the test succeeded, otherwise `false`.
    func startProxyConfigurationTest(_ completion: ((Bool) -> Void)?)

    /// Cancel currently running configuration test.
    /// The interactor is expected to reset the testing status to the initial.
    func cancelProxyConfigurationTest()
}

extension ProxyConfigurationInteractorProtocol {
    /// Start testing proxy configuration with data from view model.
    func startProxyConfigurationTest() {
        startProxyConfigurationTest(nil)
    }
}
