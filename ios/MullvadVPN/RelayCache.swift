//
//  RelayCache.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit
import os

class RelayCache {
    /// Internal procedure queue
    private let queue: ProcedureQueue = {
        let queue = ProcedureQueue()
        queue.qualityOfService = .utility
        return queue
    }()

    /// The cache location used by the class instance
    private let cacheFileURL: URL

    /// Error emitted by read and write functions
    enum Error: Swift.Error {
        case defaultLocationNotFound
        case io(Swift.Error)
        case coding(Swift.Error)
    }

    /// The default cache file location
    static var defaultCacheFileURL: URL? {
        let appGroupIdentifier = ApplicationConfiguration.securityGroupIdentifier
        let containerURL = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroupIdentifier)

        return containerURL.flatMap { URL(fileURLWithPath: "relays.json", relativeTo: $0) }
    }

    init(cacheFileURL: URL) {
        self.cacheFileURL = cacheFileURL
    }

    class func withDefaultLocation() throws -> RelayCache {
        guard let cacheFileURL = defaultCacheFileURL else {
            throw Error.defaultLocationNotFound
        }
        return RelayCache(cacheFileURL: cacheFileURL)
    }

    /// Read the relay cache and update it from remote if needed.
    /// The completion handler is called on a background queue
    func read(completion: @escaping (Result<CachedRelayList, Swift.Error>) -> Void) {
        let cacheRequestProcedure = BlockProcedure { (blockProcedure) in
            self.readAndUpdateRelaysIfNeeded(completion: { (result) in
                completion(result)
                blockProcedure.finish()
            })
        }

        cacheRequestProcedure.addCondition(MutuallyExclusive<RelayCache>())

        queue.addOperation(cacheRequestProcedure)
    }

    private func readAndUpdateRelaysIfNeeded(completion: @escaping (Result<CachedRelayList, Swift.Error>) -> Void) {
        let updateRelays = { (cachedRelaysFromDisk: CachedRelayList?,
            finish: @escaping (Result<CachedRelayList, Swift.Error>) -> Void) in
            self.downloadRelays(completion: { (result) in
                switch result {
                case .success:
                    finish(result)

                case .failure(let error):
                    os_log(.error, "Failed to update the relay cache: %s", error.localizedDescription)

                    // Return the on-disk cache in the event of networking error
                    if let cachedRelaysFromDisk = cachedRelaysFromDisk {
                        finish(.success(cachedRelaysFromDisk))
                    } else {
                        finish(result)
                    }
                }
            })
        }

        RelayCache.read(cacheFileURL: cacheFileURL) { (result) in
            switch result {
            case .success(let cachedRelays):
                if cachedRelays.needsUpdate() {
                    updateRelays(cachedRelays, completion)
                } else {
                     completion(.success(cachedRelays))
                }

            case .failure(let error):
                os_log(.error, "Failed to read the relay cache: %s", error.localizedDescription)
                updateRelays(nil,  completion)
            }
        }
    }

    private func downloadRelays(completion: @escaping (Result<CachedRelayList, Swift.Error>) -> Void) {
        // Download relays
        let downloadRelays = MullvadAPI.getRelayList()

        // Turn RelayList into CachedRelayList
        let transform = TransformProcedure { (response) -> CachedRelayList in
            let relayList = try response.result.get()

            return CachedRelayList(relayList: relayList, updatedAt: Date())
            }.injectResult(from: downloadRelays)

        // Write cache on disk
        let writeCache = AsyncTransformProcedure<CachedRelayList, CachedRelayList> { (input, finish) in
            RelayCache.write(cacheFileURL: self.cacheFileURL, record: input, completion: { (result) in
                switch result {
                case .success:
                    finish(.success(input))

                case .failure(let error):
                    finish(.failure(error))
                }
            })
            }.injectResult(from: transform)

        writeCache.addDidFinishBlockObserver { (procedure, error) in
            if let result = procedure.output.value?.into() {
                completion(result)
            } else if let error = error {
                completion(.failure(error))
            }
        }

        queue.addOperation(GroupProcedure(operations: [downloadRelays, transform, writeCache]))
    }

    /// Safely read the cache file from disk using file coordinator
    private class func read(cacheFileURL: URL, completion: @escaping (Result<CachedRelayList, Error>) -> Void) {
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) -> Void in
            var data: Data

            // Read data from disk
            do {
                data = try Data(contentsOf: fileURLForReading)
            } catch {
                completion(.failure(.io(error)))
                return
            }

            // Decode data into RelayListCacheFile
            do {
                let decoded = try JSONDecoder().decode(CachedRelayList.self, from: data)

                completion(.success(decoded))
            } catch {
                completion(.failure(.coding(error)))
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(readingItemAt: cacheFileURL,
                                   options: [.withoutChanges],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            completion(.failure(.io(error)))
        }
    }

    /// Safely write the cache file on disk using file coordinator
    private class func write(cacheFileURL: URL, record: CachedRelayList, completion: @escaping (Result<Void, Error>) -> Void) {
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForWriting: URL) -> Void in
            var data: Data

            // Encode data
            do {
                data = try JSONEncoder().encode(record)
            } catch {
                completion(.failure(.coding(error)))
                return
            }

            // Write data
            do {
                try data.write(to: fileURLForWriting)

                completion(.success(()))
            } catch {
                completion(.failure(.io(error)))
            }
        }

        var error: NSError?
        fileCoordinator.coordinate(writingItemAt: cacheFileURL,
                                   options: [.forReplacing],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            completion(.failure(.io(error)))
        }
    }
}

/// A struct that represents the relay cache on disk
struct CachedRelayList: Codable {
    /// The relay list stored within the cache entry
    var relayList: RelayList

    /// The date when this cache was last updated
    var updatedAt: Date
}

private extension CachedRelayList {
    /// Returns true if it's time to refresh the relay list cache
    func needsUpdate() -> Bool {
        let now = Date()
        guard let nextUpdate = Calendar.current.date(byAdding: .hour, value: 1, to: updatedAt) else {
            return false
        }
        return now >= nextUpdate
    }
}
