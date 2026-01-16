//
//  AppStoreMetaData.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public struct AppStoreMetaData: Decodable {
    public let version: String

    private enum ContainerKeys: CodingKey {
        case results
    }

    private enum CodingKeys: CodingKey {
        case bundleId, version
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: ContainerKeys.self)
        var array = try container.nestedUnkeyedContainer(forKey: .results)

        var version: String?
        while !array.isAtEnd {
            let app = try array.nestedContainer(keyedBy: CodingKeys.self)
            let bundleId = try app.decode(String.self, forKey: .bundleId)

            if bundleId == Bundle.main.bundleIdentifier {
                version = try app.decode(String.self, forKey: .version)
            }
        }

        if let version {
            self.version = version
        } else {
            throw DecodingError.valueNotFound(
                String.self,
                DecodingError.Context(
                    codingPath: [CodingKeys.version],
                    debugDescription: "No version found for this app"
                )
            )
        }
    }
}
