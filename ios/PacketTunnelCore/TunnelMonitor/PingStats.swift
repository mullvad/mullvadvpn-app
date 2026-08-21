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

/// Ping statistics.
struct PingStats {
    /// Dictionary holding sequence and corresponding date when echo request took place.
    var requests = [UInt16: Date]()

    /// Timestamp when last echo request was sent.
    var lastRequestDate: Date?

    /// Timestamp when last echo reply was received.
    var lastReplyDate: Date?
}
