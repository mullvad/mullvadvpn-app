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

extension Array where Element == String {
    func withCStringArray<T>(
        _ body: (UnsafePointer<UnsafePointer<CChar>?>?, UInt) -> T
    ) -> T {
        let mutablePtrs = self.compactMap { strdup($0)! }

        defer {
            for ptr in mutablePtrs {
                free(ptr)
            }
        }

        // Wrap each pointer as optional
        let optionalPtrs: [UnsafePointer<CChar>?] = mutablePtrs.map {
            UnsafePointer($0)
        }

        return optionalPtrs.withUnsafeBufferPointer { buffer in
            body(buffer.baseAddress, UInt(buffer.count))
        }
    }
}
