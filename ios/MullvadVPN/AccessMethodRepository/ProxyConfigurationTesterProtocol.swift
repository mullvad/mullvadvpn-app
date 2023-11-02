//
//  ProxyConfigurationTesterProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Type implementing access method proxy configuration testing.
protocol ProxyConfigurationTesterProtocol {
    /// Start testing proxy configuration.
    /// - Parameters:
    ///   - configuration: a proxy configuration.
    ///   - completion: a completion handler that receives `nil` upon success, otherwise the underlying error.
    func start(configuration: PersistentProxyConfiguration, completion: @escaping (Error?) -> Void)

    /// Cancel testing proxy configuration.
    func cancel()
}
