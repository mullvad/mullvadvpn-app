// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import Foundation

extension Publisher {
    /// A publisher producing a pair that contains the previous and new value.
    ///
    /// - Returns: A publisher emitting a tuple containing the previous and new value.
    func withPreviousValue() -> some Publisher<(Output?, Output), Failure> {
        return scan(nil) { ($0?.1, $1) }.compactMap { $0 }
    }
}
