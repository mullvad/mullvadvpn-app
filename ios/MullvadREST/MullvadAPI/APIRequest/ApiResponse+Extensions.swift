// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadRustRuntime

extension ApiResponse {
    public func get() throws -> (body: Data, statusCode: UInt16, sigsumDigest: String?, sigsumTimestamp: Int64?) {
        switch self {
        case .body(let body, let statusCode, let sigsumDigest, let sigsumTimestamp):
            return (body, statusCode, sigsumDigest, sigsumTimestamp)
        case .error(statusCode: let statusCode, errorDescription: let errorDescription, serverResponseCode: let serverResponseCode):
            throw APIError.init(
                statusCode: Int(statusCode),
                errorDescription: errorDescription ?? "",
                serverResponseCode: serverResponseCode)
        }
    }
}
