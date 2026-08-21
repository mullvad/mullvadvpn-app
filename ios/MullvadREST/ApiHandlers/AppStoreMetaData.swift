// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public struct AppStoreMetaData: Decodable {
    public var bundleId: String
    public var version: String
}

public struct AppStoreMetaDataResponse: Decodable {
    public let results: [AppStoreMetaData]
}
