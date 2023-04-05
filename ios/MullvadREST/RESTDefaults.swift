//
//  RESTDefaults.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-17.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    /// Default API hostname.
    public static let defaultAPIHostname = "api.mullvad.net"

    /// Default API endpoint.
    public static let defaultAPIEndpoint = AnyIPEndpoint(string: "45.83.223.196:443")!

    /// Network timeout for API requests that usually execute quickly.
    public static let defaultRequestNetworkTimeout: TimeInterval = 10

    /// Network timeout for API requests that may take time to execute.
    public static let expensiveRequestNetworkTimeout: TimeInterval = 30
}
