//
//  MaskedInputFormatterTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 10/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

class MaskedInputFormatterTests: XCTestCase {
    private static let sampleToken = "12345678"

    private static func makeConfiguration() -> MaskedInputFormatter.Configuration {
        return MaskedInputFormatter.Configuration(
            allowedInput: .numeric,
            groupSeparator: .space,
            groupSize: 4,
            shouldUseAllCaps: false
        )
    }

    func testInitialValue() {
        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: Self.makeConfiguration()
        )

        XCTAssertEqual(input.formattedString, "1234 5678")
        XCTAssertEqual(input.caretPosition, 9)
    }

    func testReplacingValue() {
        let input = MaskedInputFormatter(
            configuration: Self.makeConfiguration()
        )
        input.replace(with: "00000000")

        XCTAssertEqual(input.formattedString, "0000 0000")
        XCTAssertEqual(input.caretPosition, 9)
    }

    func testRemovingSeparator() {
        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: Self.makeConfiguration()
        )

        input.replaceCharacters(
            in: input.formattedString.range(withOffset: 4, length: 1),
            replacementString: "",
            emptySelection: true
        )

        XCTAssertEqual(input.formattedString, "1235 678")
        XCTAssertEqual(input.caretPosition, 3)
    }

    func testRemovingSeparatorRange() {
        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: Self.makeConfiguration()
        )

        input.replaceCharacters(
            in: input.formattedString.range(withOffset: 4, length: 1),
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(input.formattedString, "1234 5678")
        XCTAssertEqual(input.caretPosition, 4)
    }

    func testRemovingRange() {
        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: Self.makeConfiguration()
        )

        input.replaceCharacters(
            in: input.formattedString.range(withOffset: 7, length: 2),
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(input.formattedString, "1234 56")
        XCTAssertEqual(input.caretPosition, 7)
    }

    func testInserting() {
        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: Self.makeConfiguration()
        )

        input.replaceCharacters(
            in: input.formattedString.range(withOffset: 5, length: 0),
            replacementString: "0000",
            emptySelection: true
        )

        XCTAssertEqual(input.formattedString, "1234 0000 5678")
        XCTAssertEqual(input.caretPosition, 9)
    }

    func testReplacingRange() {
        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: Self.makeConfiguration()
        )

        input.replaceCharacters(
            in: input.formattedString.range(withOffset: 5, length: 4),
            replacementString: "0000",
            emptySelection: false
        )

        XCTAssertEqual(input.formattedString, "1234 0000")
        XCTAssertEqual(input.caretPosition, 9)
    }

    func testMaxGroups() {
        var configuration = Self.makeConfiguration()
        configuration.maxGroups = 1

        let input = MaskedInputFormatter(
            string: Self.sampleToken,
            configuration: configuration
        )

        XCTAssertEqual(input.formattedString, "1234")
        XCTAssertEqual(input.caretPosition, 4)
    }
}

private extension String {
    func range(withOffset offset: Int, length: Int) -> Range<String.Index> {
        let start = index(startIndex, offsetBy: offset)
        let end = index(start, offsetBy: length)

        return start ..< end
    }
}
