//
//  MaskedInputFormatterTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 10/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import XCTest

private let accountNumber = "12345678"

class MaskedInputFormatterTests: XCTestCase {
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
            string: accountNumber,
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
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )

        guard let range = input.formattedString.range(withOffset: 4, length: 1) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        input.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: true
        )

        XCTAssertEqual(input.formattedString, "1235 678")
        XCTAssertEqual(input.caretPosition, 3)
    }

    func testRemovingSeparatorRange() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )

        guard let range = input.formattedString.range(withOffset: 4, length: 1) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        input.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(input.formattedString, "1234 5678")
        XCTAssertEqual(input.caretPosition, 4)
    }

    func testRemovingRange() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )

        guard let range = input.formattedString.range(withOffset: 7, length: 2) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        input.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(input.formattedString, "1234 56")
        XCTAssertEqual(input.caretPosition, 7)
    }

    func testInserting() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )
        guard let range = input.formattedString.range(withOffset: 5, length: 0) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        input.replaceCharacters(
            in: range,
            replacementString: "0000",
            emptySelection: true
        )

        XCTAssertEqual(input.formattedString, "1234 0000 5678")
        XCTAssertEqual(input.caretPosition, 9)
    }

    func testReplacingRange() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )

        guard let range = input.formattedString.range(withOffset: 5, length: 4) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        input.replaceCharacters(
            in: range,
            replacementString: "0000",
            emptySelection: false
        )

        XCTAssertEqual(input.formattedString, "1234 0000")
        XCTAssertEqual(input.caretPosition, 9)
    }

    func testInvalidCharactersReplacesTextFieldTextWithFormattedString() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )
        let invalidRange = NSRange(location: accountNumber.count + 1, length: 0)
        let textField = UITextField()

        _ = input.textField(textField, shouldChangeCharactersIn: invalidRange, replacementString: "´")

        XCTAssertEqual(textField.text, input.formattedString)
    }

    func testDeleteCharacterOutsideOfTokenBoundaryDoesNotDeleteAnything() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )
        let invalidRange = NSRange(location: accountNumber.count + 1, length: 1)
        let textField = UITextField()

        _ = input.textField(textField, shouldChangeCharactersIn: invalidRange, replacementString: "")

        XCTAssertEqual(textField.text, input.formattedString)
    }

    func testDeleteLastCharacter() {
        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: Self.makeConfiguration()
        )

        let lastCharacterRange = NSRange(location: accountNumber.count, length: 1)
        let textField = UITextField()

        _ = input.textField(textField, shouldChangeCharactersIn: lastCharacterRange, replacementString: "")

        XCTAssertEqual(textField.text, "1234 567")
    }

    func testMaxGroups() {
        var configuration = Self.makeConfiguration()
        configuration.maxGroups = 1

        let input = MaskedInputFormatter(
            string: accountNumber,
            configuration: configuration
        )

        XCTAssertEqual(input.formattedString, "1234")
        XCTAssertEqual(input.caretPosition, 4)
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
