//
//  Cache.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-05-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A protocol for reading and writing to a cache file using `NSFileCoordinator` for cross process protection.
///
/// Uses `JSONDecoder` for reading and `JSONEncoder` for writing. `CacheType` must conform to `Codable`
public protocol Caching<CacheType> where CacheType: Codable {
    associatedtype CacheType

    /// The name of the cache file
    static var cacheFileName: String { get }
    /// The location of the cache file
    var cacheFileURL: URL { get }

    func readFromDisk() throws -> CacheType
    func writeToDisk(_: CacheType) throws
}

public extension Caching {
    func readFromDisk() throws -> CacheType {
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)
        let result = try fileCoordinator
            .coordinate(readingItemAt: cacheFileURL, options: [.withoutChanges]) { file in
                let data = try Data(contentsOf: file)
                let cachedFile = try JSONDecoder().decode(CacheType.self, from: data)

                return cachedFile
            }

        return result
    }

    func writeToDisk(_ cache: CacheType) throws {
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        try fileCoordinator.coordinate(writingItemAt: cacheFileURL, options: [.forReplacing]) { file in
            let data = try JSONEncoder().encode(cache)
            try data.write(to: file)
        }
    }
}
