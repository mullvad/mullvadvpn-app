// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public struct StringDecodingError: LocalizedError {
    public let data: Data

    public init(data: Data) {
        self.data = data
    }

    public var errorDescription: String? {
        "Failed to decode string from data."
    }
}

public struct StringEncodingError: LocalizedError {
    public let string: String

    public init(string: String) {
        self.string = string
    }

    public var errorDescription: String? {
        "Failed to encode string into data."
    }
}
