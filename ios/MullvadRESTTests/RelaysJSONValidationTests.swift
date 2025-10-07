//
//  RelaysJSONValidationTests.swift
//  MullvadRESTTests
//
//  Created by Test Generation on 2025-01-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest
@testable import MullvadREST

final class RelaysJSONValidationTests: XCTestCase {
    
    // MARK: - JSON Structure Tests
    
    func testRelaysJSONIsValidJSON() throws {
        // Given
        let bundle = Bundle(for: type(of: self))
        guard let url = bundle.url(forResource: "relays", withExtension: "json", subdirectory: "Assets") else {
            // Try alternative path
            let mainBundle = Bundle.main
            guard let mainUrl = mainBundle.url(forResource: "relays", withExtension: "json") else {
                XCTFail("Could not find relays.json file")
                return
            }
            let data = try Data(contentsOf: mainUrl)
            let _ = try JSONSerialization.jsonObject(with: data, options: [])
            return
        }
        
        // When
        let data = try Data(contentsOf: url)
        let jsonObject = try JSONSerialization.jsonObject(with: data, options: [])
        
        // Then
        XCTAssertNotNil(jsonObject, "JSON should be parseable")
    }
    
    func testRelaysJSONHasRequiredTopLevelKeys() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When/Then
        XCTAssertNotNil(json?["locations"], "JSON should contain 'locations' key")
        XCTAssertNotNil(json?["openvpn"], "JSON should contain 'openvpn' key")
        XCTAssertNotNil(json?["wireguard"], "JSON should contain 'wireguard' key")
        XCTAssertNotNil(json?["bridge"], "JSON should contain 'bridge' key")
    }
    
    func testLocationsAreValidDictionary() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let locations = json?["locations"] as? [String: Any] else {
            XCTFail("Locations should be a dictionary")
            return
        }
        
        // Then
        XCTAssertTrue(locations.count > 0, "Should have at least one location")
        
        // Verify each location has required fields
        for (code, locationData) in locations {
            XCTAssertFalse(code.isEmpty, "Location code should not be empty")
            
            guard let location = locationData as? [String: Any] else {
                XCTFail("Location data should be a dictionary for code: \(code)")
                continue
            }
            
            XCTAssertNotNil(location["country"], "Location should have 'country' for code: \(code)")
            XCTAssertNotNil(location["city"], "Location should have 'city' for code: \(code)")
            XCTAssertNotNil(location["latitude"], "Location should have 'latitude' for code: \(code)")
            XCTAssertNotNil(location["longitude"], "Location should have 'longitude' for code: \(code)")
        }
    }
    
    func testWireguardRelaysStructure() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let wireguard = json?["wireguard"] as? [String: Any] else {
            XCTFail("Wireguard should be a dictionary")
            return
        }
        
        // Then
        XCTAssertNotNil(wireguard["relays"], "Wireguard should have 'relays' array")
        XCTAssertNotNil(wireguard["port_ranges"], "Wireguard should have 'port_ranges'")
        XCTAssertNotNil(wireguard["ipv4_gateway"], "Wireguard should have 'ipv4_gateway'")
        XCTAssertNotNil(wireguard["ipv6_gateway"], "Wireguard should have 'ipv6_gateway'")
        
        // Verify relays array
        if let relays = wireguard["relays"] as? [[String: Any]] {
            XCTAssertTrue(relays.count > 0, "Should have at least one wireguard relay")
            
            // Verify first relay has required fields
            if let firstRelay = relays.first {
                XCTAssertNotNil(firstRelay["hostname"], "Relay should have hostname")
                XCTAssertNotNil(firstRelay["location"], "Relay should have location")
                XCTAssertNotNil(firstRelay["active"], "Relay should have active status")
                XCTAssertNotNil(firstRelay["public_key"], "Relay should have public_key")
                XCTAssertNotNil(firstRelay["ipv4_addr_in"], "Relay should have ipv4_addr_in")
            }
        }
    }
    
    func testOpenVPNRelaysStructure() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let openvpn = json?["openvpn"] as? [String: Any] else {
            XCTFail("OpenVPN should be a dictionary")
            return
        }
        
        // Then
        XCTAssertNotNil(openvpn["relays"], "OpenVPN should have 'relays' array")
        XCTAssertNotNil(openvpn["ports"], "OpenVPN should have 'ports' array")
        
        // Verify ports structure
        if let ports = openvpn["ports"] as? [[String: Any]] {
            XCTAssertTrue(ports.count > 0, "Should have at least one port configuration")
            
            for port in ports {
                XCTAssertNotNil(port["port"], "Port config should have 'port'")
                XCTAssertNotNil(port["protocol"], "Port config should have 'protocol'")
            }
        }
    }
    
    func testBridgeRelaysStructure() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let bridge = json?["bridge"] as? [String: Any] else {
            XCTFail("Bridge should be a dictionary")
            return
        }
        
        // Then
        XCTAssertNotNil(bridge["relays"], "Bridge should have 'relays' array")
        XCTAssertNotNil(bridge["shadowsocks"], "Bridge should have 'shadowsocks' array")
    }
    
    // MARK: - Data Validation Tests
    
    func testRelayIPAddressesAreValid() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let wireguard = json?["wireguard"] as? [String: Any],
              let relays = wireguard["relays"] as? [[String: Any]] else {
            XCTFail("Could not parse wireguard relays")
            return
        }
        
        // Then - Check a few relays for valid IP addresses
        let relaysToCheck = Array(relays.prefix(5))
        for relay in relaysToCheck {
            if let ipv4 = relay["ipv4_addr_in"] as? String {
                XCTAssertTrue(isValidIPv4(ipv4), "IPv4 address should be valid: \(ipv4)")
            }
        }
    }
    
    func testLocationCoordinatesAreValid() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let locations = json?["locations"] as? [String: Any] else {
            XCTFail("Could not parse locations")
            return
        }
        
        // Then - Check coordinates are within valid ranges
        for (code, locationData) in locations {
            guard let location = locationData as? [String: Any] else { continue }
            
            if let lat = location["latitude"] as? Double {
                XCTAssertTrue(lat >= -90 && lat <= 90,
                            "Latitude should be between -90 and 90 for \(code): \(lat)")
            }
            
            if let lon = location["longitude"] as? Double {
                XCTAssertTrue(lon >= -180 && lon <= 180,
                            "Longitude should be between -180 and 180 for \(code): \(lon)")
            }
        }
    }
    
    func testPortRangesAreValid() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let wireguard = json?["wireguard"] as? [String: Any],
              let portRanges = wireguard["port_ranges"] as? [[Int]] else {
            XCTFail("Could not parse port ranges")
            return
        }
        
        // Then
        XCTAssertTrue(portRanges.count > 0, "Should have at least one port range")
        
        for range in portRanges {
            XCTAssertEqual(range.count, 2, "Port range should have exactly 2 values")
            if range.count == 2 {
                let start = range[0]
                let end = range[1]
                XCTAssertTrue(start >= 1 && start <= 65535, "Start port should be valid")
                XCTAssertTrue(end >= 1 && end <= 65535, "End port should be valid")
                XCTAssertLessThanOrEqual(start, end, "Start port should be <= end port")
            }
        }
    }
    
    func testActiveRelaysAreBoolean() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let wireguard = json?["wireguard"] as? [String: Any],
              let relays = wireguard["relays"] as? [[String: Any]] else {
            XCTFail("Could not parse relays")
            return
        }
        
        // Then
        for relay in relays.prefix(10) {
            if let active = relay["active"] {
                XCTAssertTrue(active is Bool, "Active field should be boolean for \(relay["hostname"] ?? "unknown")")
            }
        }
    }
    
    func testWeightsAreNumeric() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let wireguard = json?["wireguard"] as? [String: Any],
              let relays = wireguard["relays"] as? [[String: Any]] else {
            XCTFail("Could not parse relays")
            return
        }
        
        // Then
        for relay in relays.prefix(10) {
            if let weight = relay["weight"] {
                XCTAssertTrue(weight is Int || weight is Double,
                            "Weight should be numeric for \(relay["hostname"] ?? "unknown")")
                if let weightValue = weight as? Int {
                    XCTAssertGreaterThanOrEqual(weightValue, 0, "Weight should be non-negative")
                }
            }
        }
    }
    
    // MARK: - Consistency Tests
    
    func testAllRelayLocationsExist() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        guard let locations = json?["locations"] as? [String: Any],
              let wireguard = json?["wireguard"] as? [String: Any],
              let relays = wireguard["relays"] as? [[String: Any]] else {
            XCTFail("Could not parse JSON structure")
            return
        }
        
        let locationCodes = Set(locations.keys)
        
        // When/Then - Check first 20 relays
        for relay in relays.prefix(20) {
            if let location = relay["location"] as? String {
                XCTAssertTrue(
                    locationCodes.contains(location),
                    "Relay location '\(location)' should exist in locations dictionary"
                )
            }
        }
    }
    
    func testNoEmptyHostnames() throws {
        // Given
        let jsonData = try loadRelaysJSON()
        let json = try JSONSerialization.jsonObject(with: jsonData) as? [String: Any]
        
        // When
        guard let wireguard = json?["wireguard"] as? [String: Any],
              let relays = wireguard["relays"] as? [[String: Any]] else {
            XCTFail("Could not parse relays")
            return
        }
        
        // Then
        for relay in relays {
            if let hostname = relay["hostname"] as? String {
                XCTAssertFalse(hostname.isEmpty, "Hostname should not be empty")
            } else {
                XCTFail("Relay missing hostname")
            }
        }
    }
    
    // MARK: - Helper Methods
    
    private func loadRelaysJSON() throws -> Data {
        // Try to load from test bundle
        let testBundle = Bundle(for: type(of: self))
        
        // Try various possible paths
        let possiblePaths = [
            testBundle.url(forResource: "relays", withExtension: "json", subdirectory: "Assets"),
            testBundle.url(forResource: "relays", withExtension: "json"),
            Bundle.main.url(forResource: "relays", withExtension: "json")
        ]
        
        for path in possiblePaths {
            if let url = path {
                if let data = try? Data(contentsOf: url) {
                    return data
                }
            }
        }
        
        // If we can't find the file in bundles, try to construct it from the repository path
        let fileManager = FileManager.default
        let currentPath = fileManager.currentDirectoryPath
        let relaysPath = "\(currentPath)/ios/MullvadREST/Assets/relays.json"
        
        if fileManager.fileExists(atPath: relaysPath) {
            return try Data(contentsOf: URL(fileURLWithPath: relaysPath))
        }
        
        throw NSError(domain: "TestError", code: 1, userInfo: [NSLocalizedDescriptionKey: "Could not find relays.json"])
    }
    
    private func isValidIPv4(_ ip: String) -> Bool {
        let parts = ip.split(separator: ".")
        guard parts.count == 4 else { return false }
        
        return parts.allSatisfy { part in
            guard let num = Int(part) else { return false }
            return num >= 0 && num <= 255
        }
    }
}