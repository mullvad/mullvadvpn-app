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
import MullvadRustRuntime
import MullvadTypes

extension REST {
    /// The API hostname and endpoint are defined in the Info.plist of the MullvadREST framework bundle
    /// This is due to not being able to target `Bundle.main` from a Unit Test environment as it gets its own bundle that would not contain the above variables.
    nonisolated(unsafe) private static let infoDictionary = Bundle(for: ProxyFactory.self).infoDictionary!

    /// Default API hostname.
    public static let defaultAPIHostname = infoDictionary["ApiHostName"] as! String

    /// Default API endpoint.
    public static let defaultAPIEndpoint = AnyIPEndpoint(string: infoDictionary["ApiEndpoint"] as! String)!

    public static let encryptedDNSHostname = infoDictionary["EncryptedDnsHostName"] as! String

    /// Disables API IP address cache when in staging environment and sticks to using default API endpoint instead.
    public static let isStagingEnvironment = false

    /// Default network timeout for API requests.
    public static let defaultAPINetworkTimeout: Duration = .seconds(10)

    /// am.i.mullvad.net hostname.
    public static let amIMullvadHostname = infoDictionary["AmIMullvad"] as! String
}
