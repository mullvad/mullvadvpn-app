//
//  AppStoreMetaData.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public struct AppStoreMetaData: Decodable {
    public var bundleId: String
    public var version: String
}

public struct AppStoreMetaDataResponse: Decodable {
    public let results: [AppStoreMetaData]
}
