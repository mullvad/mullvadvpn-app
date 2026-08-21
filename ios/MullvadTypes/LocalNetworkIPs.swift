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

public enum LocalNetworkIPs: String {
    case gatewayAddressIpV4 = "10.64.0.1"
    case gatewayAddressIpV6 = "fc00::1"
    case defaultRouteIpV4 = "0.0.0.0"
    case defaultRouteIpV6 = "::"
}
