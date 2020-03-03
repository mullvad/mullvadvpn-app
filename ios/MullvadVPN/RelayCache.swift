//
//  RelayCache.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Combine
import os

/// Error emitted by read and write functions
enum RelayCacheError: Error {
    case defaultLocationNotFound
    case io(Error)
    case coding(Error)
    case network(MullvadAPI.Error)
    case server(JsonRpcResponseError<MullvadAPI.ResponseCode>)
}

/// A enum describing the source of the relay list
enum RelayListSource {
    /// The relay list was received from network
    case network

    /// The relay list was read from cache
    case cache
}

class RelayCache {

    /// Mullvad API client
    private let apiClient: MullvadAPI

    /// The cache location used by the class instance
    private let cacheFileURL: URL

    /// A queue used for running cache requests that require mutual exclusivity
    private let exclusivityQueue = DispatchQueue(label: "net.mullvad.vpn.relay-cache.exclusivity-queue")

    /// A queue used for execution
    private let executionQueue = DispatchQueue(label: "net.mullvad.vpn.relay-cache.execution-queue")

    /// The default cache file location
    static var defaultCacheFileURL: URL? {
        let appGroupIdentifier = ApplicationConfiguration.securityGroupIdentifier
        let containerURL = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroupIdentifier)

        return containerURL.flatMap { URL(fileURLWithPath: "relays.json", relativeTo: $0) }
    }

    init(cacheFileURL: URL, networkSession: URLSession = URLSession.shared) {
        apiClient = MullvadAPI(session: networkSession)
        self.cacheFileURL = cacheFileURL
    }

    class func withDefaultLocation() -> Result<RelayCache, RelayCacheError> {
        if let cacheFileURL = defaultCacheFileURL {
            return .success(RelayCache(cacheFileURL: cacheFileURL))
        } else {
            return .failure(.defaultLocationNotFound)
        }
    }

    /// Read the relay cache and update it from remote if needed.
    func read() -> AnyPublisher<CachedRelayList, RelayCacheError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            self.makeReaderPublisher()
        }.eraseToAnyPublisher()
    }

    private func makeReaderPublisher() -> AnyPublisher<CachedRelayList, RelayCacheError> {
        // Create a deferred publisher that will execute once the subscriber is assigned
        let downloadAndSaveRelaysPublisher = Deferred {
            return self.downloadRelays()
                .map(self.filterRelayList)
                .flatMap(self.saveRelayListToCache)
                .mapError { (error) -> RelayCacheError in
                    os_log(.error, "Failed to update the relay cache: %{public}s", error.localizedDescription)

                    return error
            }
        }

        return Self.read(cacheFileURL: cacheFileURL).publisher
            .map { (RelayListSource.cache, $0) }
            .catch({ (readError) -> AnyPublisher<(RelayListSource, CachedRelayList), RelayCacheError> in
                switch readError {
                // Download relay list when unable to read the cache file
                case .io(let error as CocoaError) where error.code == .fileReadNoSuchFile:
                    os_log(.error, "Relay cache file does not exist. Initiating the download.")

                    return downloadAndSaveRelaysPublisher.map { (RelayListSource.network, $0) }
                        .eraseToAnyPublisher()

                case .coding(let decodingError):
                    os_log(.error, "Failed to decode the relay cache: %{public}s", decodingError.localizedDescription)

                    return downloadAndSaveRelaysPublisher.map { (RelayListSource.network, $0) }
                        .eraseToAnyPublisher()

                default:
                    os_log(.error, "Failed to read the relay cache: %{public}s", readError.localizedDescription)

                    return Fail(error: readError).eraseToAnyPublisher()
                }
            })
            .flatMap { (source, cachedRelays) -> AnyPublisher<CachedRelayList, RelayCacheError> in
                let cachedRelayPublisher = Result<CachedRelayList, RelayCacheError>.Publisher(cachedRelays)

                if source == .cache && cachedRelays.needsUpdate() {
                    return downloadAndSaveRelaysPublisher
                        .catch { (error) -> Result<CachedRelayList, RelayCacheError>.Publisher in
                            // Return the on-disk cache in the event of networking error
                            return cachedRelayPublisher
                    }.eraseToAnyPublisher()
                } else {
                    return cachedRelayPublisher
                        .eraseToAnyPublisher()
                }
        }.eraseToAnyPublisher()
    }

    /// Filters the given `RelayList` removing empty leaf nodes, relays without Wireguard tunnels or
    /// Wireguard tunnels without any available ports.
    private func filterRelayList(_ relayList: RelayList) -> RelayList {
        let filteredCountries = relayList.countries
            .map { (country) -> RelayList.Country in
                var filteredCountry = country

                filteredCountry.cities = country.cities.map { (city) -> RelayList.City in
                    var filteredCity = city

                    filteredCity.relays = city.relays
                        .map { (relay) -> RelayList.Relay in
                            var filteredRelay = relay

                            // filter out tunnels without ports
                            filteredRelay.tunnels?.wireguard = relay.tunnels?.wireguard?
                                .filter { !$0.portRanges.isEmpty }

                            return filteredRelay
                    }.filter { $0.tunnels?.wireguard.flatMap { !$0.isEmpty } ?? false }

                    return filteredCity
                }.filter { !$0.relays.isEmpty }

                return filteredCountry
        }.filter { !$0.cities.isEmpty }

        return RelayList(countries: filteredCountries)
    }

    private func downloadRelays() -> AnyPublisher<RelayList, RelayCacheError> {
        apiClient.getRelayList()
            .mapError({ (networkError) -> RelayCacheError in
                return .network(networkError)
            })
            .flatMap({ (response) in
                return response.result.publisher
                    .mapError { RelayCacheError.server($0) }
            }).eraseToAnyPublisher()
    }

    private func saveRelayListToCache(relayList: RelayList) -> AnyPublisher<CachedRelayList, RelayCacheError> {
        Result.Publisher(relayList)
            .map({ CachedRelayList(relayList: $0, updatedAt: Date()) })
            .flatMap({ (cachedRelayList) in
                return Self.write(cacheFileURL: self.cacheFileURL, record: cachedRelayList)
                    .map { cachedRelayList }
                    .publisher
            }).eraseToAnyPublisher()
    }

    /// Safely read the cache file from disk using file coordinator
    private class func read(cacheFileURL: URL) -> Result<CachedRelayList, RelayCacheError> {
        var result: Result<CachedRelayList, RelayCacheError>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) -> Void in
            // Decode data from disk
            result = Result { try Data(contentsOf: fileURLForReading) }
                .mapError { RelayCacheError.io($0) }
                .flatMap { (data) in
                    Result { try JSONDecoder().decode(CachedRelayList.self, from: data) }
                        .mapError { RelayCacheError.coding($0) }
                }
        }

        var error: NSError?
        fileCoordinator.coordinate(readingItemAt: cacheFileURL,
                                   options: [.withoutChanges],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            result = .failure(.io(error))
        }

        return result!
    }

    /// Safely write the cache file on disk using file coordinator
    private class func write(cacheFileURL: URL, record: CachedRelayList) -> Result<(), RelayCacheError> {
        var result: Result<(), RelayCacheError>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForWriting: URL) -> Void in
            result = Result { try JSONEncoder().encode(record) }
                .mapError { RelayCacheError.coding($0) }
                .flatMap { (data) in
                    Result { try data.write(to: fileURLForWriting) }
                        .mapError { RelayCacheError.io($0) }
                }
        }

        var error: NSError?
        fileCoordinator.coordinate(writingItemAt: cacheFileURL,
                                   options: [.forReplacing],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            result = .failure(.io(error))
        }

        return result!
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
