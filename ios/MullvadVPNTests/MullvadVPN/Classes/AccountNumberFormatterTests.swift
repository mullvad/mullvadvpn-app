//
//  AccountNumberFormatterTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Testing

struct AccountNumberFormatterTests {
    private let formatter = GroupedTextFormatter.accountNumber

    @Test
    func testInitialValue() {
        var formattedString = formatter.format("12345678")
        #expect(formattedString == "1234 5678")

        formattedString = formatter.format("12345678123456781")
        #expect(formattedString == "1234 5678 1234 5678")

        formattedString = formatter.format("12XB345")
        #expect(formattedString == "1234 5")
    }

    @Test
    func testReplacingValue() {
        let value = "123456789"
        var formattedString = formatter.format(value)
        formattedString.replaceSubrange(
            formattedString.startIndex...formattedString.index(formattedString.startIndex, offsetBy: 3), with: "90")
        #expect(formatter.format(formattedString) == "9056 789")
    }

    @Test
    func testInserting() {
        let value = "123456789"
        var formattedString = formatter.format(value)
        formattedString.insert(
            contentsOf: "0000",
            at: formattedString.index(after: formattedString.index(formattedString.startIndex, offsetBy: 2)))

        #expect(formatter.format(formattedString) == "1230 0004 5678 9")
    }

    @Test
    func testInsertingInvalid() {
        let value = "123456789"
        var formattedString = formatter.format(value)
        formattedString.insert(
            contentsOf: "xy",
            at: formattedString.index(after: formattedString.index(formattedString.startIndex, offsetBy: 2)))

        #expect(formatter.format(formattedString) == "1234 5678 9")
    }
}
