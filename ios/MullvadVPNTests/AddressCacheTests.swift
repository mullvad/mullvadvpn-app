//
//  AddressCacheTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2023-05-05.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadTypes
import struct Network.IPv4Address
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
        addressCache.setEndpoints([apiEndpoint])
        let dateAfterUpdate = addressCache.getLastUpdateDate()

        XCTAssertNotEqual(dateBeforeUpdate, dateAfterUpdate)
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

        let fileCache = MockFileCache<REST.CachedAddresses>()
        let addressCache = REST.AddressCache(canWriteToCache: true, fileCache: fileCache)
        addressCache.setEndpoints(ipAddresses)

        let fileState = fileCache.getState()
        guard case let .exists(cachedAddresses) = fileState else {
            XCTFail("State is expected to contain cached addresses.")
            return
        }

        XCTAssertEqual(cachedAddresses.endpoint, firstIPEndpoint)
        XCTAssertEqual(addressCache.getCurrentEndpoint(), firstIPEndpoint)
    }

    func testCacheReadsFromFile() throws {
        let fixedDate = Date()
        let addressCache = REST.AddressCache(
            canWriteToCache: true,
            fileCache: MockFileCache(initialState: .exists(
                REST.CachedAddresses(updatedAt: fixedDate, endpoint: apiEndpoint)
            ))
        )
        addressCache.loadFromFile()

        XCTAssertEqual(addressCache.getCurrentEndpoint(), apiEndpoint)
        XCTAssertEqual(addressCache.getLastUpdateDate(), fixedDate)
    }

    func testCacheWritesToDiskWhenSettingNewEndpoints() throws {
        let fileCache = MockFileCache<REST.CachedAddresses>()
        let addressCache = REST.AddressCache(canWriteToCache: true, fileCache: fileCache)

        XCTAssertEqual(fileCache.getState(), .fileNotFound)
        addressCache.setEndpoints([apiEndpoint])

        let fileState = fileCache.getState()
        XCTAssertTrue(fileState.isExists)

        guard case let .exists(cachedAddresses) = fileState else {
            XCTFail("State is expected to contain cached addresses.")
            return
        }

        XCTAssertEqual(cachedAddresses.endpoint, addressCache.getCurrentEndpoint())
        XCTAssertEqual(cachedAddresses.updatedAt, addressCache.getLastUpdateDate())
    }

    func testGetCurrentEndpointReadsFromCacheWhenReadOnly() throws {
        let addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .exists(
                REST.CachedAddresses(updatedAt: Date(), endpoint: apiEndpoint)
            ))
        )
        XCTAssertEqual(addressCache.getCurrentEndpoint(), apiEndpoint)
    }
}
