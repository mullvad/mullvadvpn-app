//
//  RESTDefaults.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-17.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

// swiftlint:disable force_cast
extension REST {
    /// The API hostname and endpoint are defined in the Info.plist of the MullvadREST framework bundle
    /// This is due to not being able to target `Bundle.main` from a Unit Test environment as it gets its own bundle that would not contain the above variables.
    private static let infoDictionary = Bundle(for: AddressCache.self).infoDictionary!

    /// Default API hostname.
    public static let defaultAPIHostname = infoDictionary["ApiHostName"] as! String

    /// Default API endpoint.
    public static let defaultAPIEndpoint = AnyIPEndpoint(string: infoDictionary["ApiEndpoint"] as! String)!

    /// Disables API IP address cache when in staging environment and sticks to using default API endpoint instead.
    public static let isStagingEnvironment = false

    /// Default network timeout for API requests.
    public static let defaultAPINetworkTimeout: Duration = .seconds(10)
}

// swiftlint:enable force_cast
