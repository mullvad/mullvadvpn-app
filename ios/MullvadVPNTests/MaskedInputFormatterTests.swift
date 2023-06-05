//
//  MaskedInputFormatterTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 10/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

class MaskedInputFormatterTests: XCTestCase {
    private let accountNumber = "12345678"
    private var subjectUnderTest: MaskedInputFormatter!

    private var configuration: MaskedInputFormatter.Configuration = .init(
        allowedInput: .numeric,
        groupSeparator: " ",
        groupSize: 4,
        maxGroups: 4
    )

    override func setUp() {
        subjectUnderTest = .init(
            string: accountNumber,
            configuration: configuration
        )
    }

    override func tearDown() {
        subjectUnderTest = nil
    }

    func testInitialValue() {
        XCTAssertEqual(subjectUnderTest.formattedString, "1234 5678")
        XCTAssertEqual(subjectUnderTest.caretPosition, 9)
    }

    func testReplacingValue() {
        subjectUnderTest.replace(with: "00000000")

        XCTAssertEqual(subjectUnderTest.formattedString, "0000 0000")
        XCTAssertEqual(subjectUnderTest.caretPosition, 9)
    }

    func testRemovingSeparator() {
        guard let range = subjectUnderTest.formattedString.range(withOffset: 4, length: 1) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        subjectUnderTest.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: true
        )

        XCTAssertEqual(subjectUnderTest.formattedString, "1235 678")
        XCTAssertEqual(subjectUnderTest.caretPosition, 3)
    }

    func testRemovingSeparatorRange() {
        guard let range = subjectUnderTest.formattedString.range(withOffset: 4, length: 1) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        subjectUnderTest.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(subjectUnderTest.formattedString, "1234 5678")
        XCTAssertEqual(subjectUnderTest.caretPosition, 4)
    }

    func testRemovingRange() {
        guard let range = subjectUnderTest.formattedString.range(withOffset: 7, length: 2) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        subjectUnderTest.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(subjectUnderTest.formattedString, "1234 56")
        XCTAssertEqual(subjectUnderTest.caretPosition, 7)
    }

    func testInserting() {
        guard let range = subjectUnderTest.formattedString.range(withOffset: 5, length: 0) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        subjectUnderTest.replaceCharacters(
            in: range,
            replacementString: "0000",
            emptySelection: true
        )

        XCTAssertEqual(subjectUnderTest.formattedString, "1234 0000 5678")
        XCTAssertEqual(subjectUnderTest.caretPosition, 9)
    }

    func testReplacingRange() {
        guard let range = subjectUnderTest.formattedString.range(withOffset: 5, length: 4) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        subjectUnderTest.replaceCharacters(
            in: range,
            replacementString: "0000",
            emptySelection: false
        )

        XCTAssertEqual(subjectUnderTest.formattedString, "1234 0000")
        XCTAssertEqual(subjectUnderTest.caretPosition, 9)
    }

    func testInvalidCharactersReplacesTextFieldTextWithFormattedString() {
        let invalidRange = NSRange(location: accountNumber.count + 1, length: 0)
        let textField = UITextField()

        _ = subjectUnderTest.textField(textField, shouldChangeCharactersIn: invalidRange, replacementString: "´")

        XCTAssertEqual(textField.text, subjectUnderTest.formattedString)
    }

    func testDeleteCharacterOutsideOfTokenBoundaryDoesNotDeleteAnything() {
        let invalidRange = NSRange(location: accountNumber.count + 1, length: 1)
        let textField = UITextField()

        _ = subjectUnderTest.textField(textField, shouldChangeCharactersIn: invalidRange, replacementString: "")

        XCTAssertEqual(textField.text, subjectUnderTest.formattedString)
    }

    func testDeleteLastCharacter() {
        let lastCharacterRange = NSRange(location: accountNumber.count, length: 1)
        let textField = UITextField()

        _ = subjectUnderTest.textField(textField, shouldChangeCharactersIn: lastCharacterRange, replacementString: "")

        XCTAssertEqual(textField.text, "1234 567")
    }
}

private extension String {
    func range(withOffset offset: Int, length: Int) -> Range<String.Index>? {
        guard let start = index(startIndex, offsetBy: offset, limitedBy: endIndex),
              let end = index(start, offsetBy: length, limitedBy: endIndex)
        else {
            return nil
        }
        return start ..< end
    }
}
