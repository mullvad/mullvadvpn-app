//
//  AccountTokenInputTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 10/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

private let kSampleToken = "12345678"

class AccountTokenInputTests: XCTestCase {
    var sut: InputFormatter!

    override func setUp() {
        sut = InputFormatter(
            allowedInput: .numerical,
            groupSeparator: .space
        )
    }

    override func tearDown() {
        sut = nil
    }

    func testInitialValue() {
        sut.replace(with: kSampleToken)

        XCTAssertEqual(sut.formattedString, "1234 5678")
        XCTAssertEqual(sut.caretPosition, 9)
    }

    func testReplacingValue() {
        sut.replace(with: "00000000")

        XCTAssertEqual(sut.formattedString, "0000 0000")
        XCTAssertEqual(sut.caretPosition, 9)
    }

    func testRemovingSeparator() {
        sut.replace(with: kSampleToken)

        sut.replaceCharacters(
            in: sut.formattedString.range(withOffset: 4, length: 1),
            replacementString: "",
            emptySelection: true
        )

        XCTAssertEqual(sut.formattedString, "1235 678")
        XCTAssertEqual(sut.caretPosition, 3)
    }

    func testRemovingSeparatorRange() {
        sut.replace(with: kSampleToken)

        sut.replaceCharacters(
            in: sut.formattedString.range(withOffset: 4, length: 1),
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(sut.formattedString, "1234 5678")
        XCTAssertEqual(sut.caretPosition, 4)
    }

    func testRemovingRange() {
        sut.replace(with: kSampleToken)

        sut.replaceCharacters(
            in: sut.formattedString.range(withOffset: 7, length: 2),
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(sut.formattedString, "1234 56")
        XCTAssertEqual(sut.caretPosition, 7)
    }

    func testInserting() {
        sut.replace(with: kSampleToken)

        sut.replaceCharacters(
            in: sut.formattedString.range(withOffset: 5, length: 0),
            replacementString: "0000",
            emptySelection: true
        )

        XCTAssertEqual(sut.formattedString, "1234 0000 5678")
        XCTAssertEqual(sut.caretPosition, 9)
    }

    func testReplacingRange() {
        sut.replace(with: kSampleToken)

        sut.replaceCharacters(
            in: sut.formattedString.range(withOffset: 5, length: 4),
            replacementString: "0000",
            emptySelection: false
        )

        XCTAssertEqual(sut.formattedString, "1234 0000")
        XCTAssertEqual(sut.caretPosition, 9)
    }
}

private extension String {
    func range(withOffset offset: Int, length: Int) -> Range<String.Index> {
        let start = index(startIndex, offsetBy: offset)
        let end = index(start, offsetBy: length)

        return start ..< end
    }
}
