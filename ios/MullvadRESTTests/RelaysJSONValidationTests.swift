//
//  RelaysJSONValidationTests.swift
//  MullvadRESTTests
//
//  Created by Test Suite on 2025-10-07.
//

import Foundation
import Testing

@testable import MullvadREST

@Suite("relays.json validation")
struct RelaysJSONValidationTests {
    @Test("relays.json is parseable and has basic keys")
    func testRelaysJSONIsValid() throws {
        guard let url = findRelaysJSON() else {
            Issue.record("relays.json not found in available bundles; skipping structural validation")
            return
        }
        let data = try Data(contentsOf: url)
        let jsonAny = try JSONSerialization.jsonObject(with: data, options: [])
        guard let json = jsonAny as? [String: Any] else {
            Issue.record("Root JSON is not a dictionary")
            return
        }
        #expect(json["locations"] \!= nil)
        #expect(json["wireguard"] \!= nil)
        #expect(json["bridge"] \!= nil)
        #expect(json["openvpn"] \!= nil)
    }
}

// Look for the asset in main/framework bundles to avoid hardcoding paths
private func findRelaysJSON() -> URL? {
    let bundles = [Bundle.main] + Bundle.allBundles + Bundle.allFrameworks
    for b in bundles {
        if let u = b.url(forResource: "relays", withExtension: "json", subdirectory: "Assets") { return u }
        if let u = b.url(forResource: "relays", withExtension: "json") { return u }
    }
    return nil
}