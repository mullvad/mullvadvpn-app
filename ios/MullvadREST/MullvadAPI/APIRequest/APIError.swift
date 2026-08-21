// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public struct APIError: Error, Codable, Sendable, CustomDebugStringConvertible {
    public let statusCode: Int
    public let errorDescription: String
    public let serverResponseCode: String?

    public init(statusCode: Int, errorDescription: String, serverResponseCode: String?) {
        self.statusCode = statusCode
        self.errorDescription = errorDescription
        self.serverResponseCode = serverResponseCode
    }

    public var debugDescription: String {
        "\(statusCode): \(errorDescription)"
    }
}
