//
//  AddressCacheTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-05-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadTypes
import Network
import XCTest

final class AddressCacheTests: XCTestCase {
    let apiEndpoint: AnyIPEndpoint = .ipv4(IPv4Endpoint(ip: IPv4Address.loopback, port: 80))

    // MARK: - Tests

    func testAddressCacheHasDefaultEndpoint() {
        let addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .fileNotFound)
        )
        XCTAssertEqual(addressCache.getCurrentEndpoint(), REST.defaultAPIEndpoint)
    }

    func testSetEndpoints() throws {
        let addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .fileNotFound)
        )

        addressCache.setEndpoints([apiEndpoint])
        XCTAssertEqual(addressCache.getCurrentEndpoint(), apiEndpoint)
    }

    func testSetEndpointsUpdatesDateWhenSettingSameAddress() throws {
        let addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .fileNotFound)
        )
        addressCache.setEndpoints([apiEndpoint])

        let dateBeforeUpdate = addressCache.getLastUpdateDate()
        // Calling `Date()` several times in a row can result in the same Date object being returned.
        // Force a sleep before setting the next endpoint to avoid getting the same Date object twice in a row.
        Thread.sleep(forTimeInterval: Duration.milliseconds(10).timeInterval)
        addressCache.setEndpoints([apiEndpoint])
        let dateAfterUpdate = addressCache.getLastUpdateDate()

        let timeDifference = dateAfterUpdate.timeIntervalSince(dateBeforeUpdate)
        XCTAssertNotEqual(0.0, timeDifference)
    }

    func testSetEndpointsDoesNotDoAnythingIfSettingEmptyEndpoints() throws {
        let addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .fileNotFound)
        )
        addressCache.loadFromFile()

        let currentEndpoint = addressCache.getCurrentEndpoint()
        addressCache.setEndpoints([])

        XCTAssertEqual(addressCache.getCurrentEndpoint(), currentEndpoint)
    }

    func testSetEndpointsOnlyAcceptsTheFirstEndpoint() throws {
        let ipAddresses = (1 ... 10)
            .map { "\($0).\($0).\($0).\($0):80" }
            .compactMap { AnyIPEndpoint(string: $0) }

        let firstIPEndpoint = try XCTUnwrap(ipAddresses.first)

        let fileCache = MockFileCache<REST.StoredAddressCache>()
        let addressCache = REST.AddressCache(canWriteToCache: true, fileCache: fileCache)
        addressCache.setEndpoints(ipAddresses)

        let fileState = fileCache.getState()
        guard case let .exists(storedAddressCache) = fileState else {
            XCTFail("State is expected to contain cached addresses.")
            return
        }

        XCTAssertEqual(storedAddressCache.endpoint, firstIPEndpoint)
        XCTAssertEqual(addressCache.getCurrentEndpoint(), firstIPEndpoint)
    }

    func testCacheReadsFromFile() throws {
        let fixedDate = Date()
        let addressCache = REST.AddressCache(
            canWriteToCache: true,
            fileCache: MockFileCache(initialState: .exists(
                REST.StoredAddressCache(updatedAt: fixedDate, endpoint: apiEndpoint)
            ))
        )
        addressCache.loadFromFile()

        XCTAssertEqual(addressCache.getCurrentEndpoint(), apiEndpoint)
        XCTAssertEqual(addressCache.getLastUpdateDate(), fixedDate)
    }

    func testCacheWritesToDiskWhenSettingNewEndpoints() throws {
        let fileCache = MockFileCache<REST.StoredAddressCache>()
        let addressCache = REST.AddressCache(canWriteToCache: true, fileCache: fileCache)

        XCTAssertEqual(fileCache.getState(), .fileNotFound)
        addressCache.setEndpoints([apiEndpoint])

        let fileState = fileCache.getState()
        XCTAssertTrue(fileState.isExists)

        guard case let .exists(storedAddressCache) = fileState else {
            XCTFail("State is expected to contain cached addresses.")
            return
        }

        XCTAssertEqual(storedAddressCache.endpoint, addressCache.getCurrentEndpoint())
        XCTAssertEqual(storedAddressCache.updatedAt, addressCache.getLastUpdateDate())
    }

    func testGetCurrentEndpointReadsFromCacheWhenReadOnly() throws {
        let addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .exists(
                REST.StoredAddressCache(updatedAt: Date(), endpoint: apiEndpoint)
            ))
        )
        XCTAssertEqual(addressCache.getCurrentEndpoint(), apiEndpoint)
    }
}
