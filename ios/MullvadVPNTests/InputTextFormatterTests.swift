//
//  InputTextFormatterTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 10/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadVPN
import XCTest

class InputTextFormatterTests: XCTestCase {
    private let accountNumber = "12345678"
    private var inputTextFormatter: InputTextFormatter!

    private let configuration = InputTextFormatter.Configuration(
        allowedInput: .numeric,
        groupSeparator: " ",
        groupSize: 4,
        maxGroups: 4
    )

    override func setUp() {
        inputTextFormatter = InputTextFormatter(
            string: accountNumber,
            configuration: configuration
        )
    }

    override func tearDown() {
        inputTextFormatter = nil
    }

    func testInitialValue() {
        XCTAssertEqual(inputTextFormatter.formattedString, "1234 5678")
        XCTAssertEqual(inputTextFormatter.caretPosition, 9)
    }

    func testReplacingValue() {
        inputTextFormatter.replace(with: "00000000")

        XCTAssertEqual(inputTextFormatter.formattedString, "0000 0000")
        XCTAssertEqual(inputTextFormatter.caretPosition, 9)
    }

    func testRemovingSeparator() {
        guard let range = inputTextFormatter.formattedString.range(withOffset: 4, length: 1) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        inputTextFormatter.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: true
        )

        XCTAssertEqual(inputTextFormatter.formattedString, "1235 678")
        XCTAssertEqual(inputTextFormatter.caretPosition, 3)
    }

    func testRemovingSeparatorRange() {
        guard let range = inputTextFormatter.formattedString.range(withOffset: 4, length: 1) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        inputTextFormatter.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(inputTextFormatter.formattedString, "1234 5678")
        XCTAssertEqual(inputTextFormatter.caretPosition, 4)
    }

    func testRemovingRange() {
        guard let range = inputTextFormatter.formattedString.range(withOffset: 7, length: 2) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        inputTextFormatter.replaceCharacters(
            in: range,
            replacementString: "",
            emptySelection: false
        )

        XCTAssertEqual(inputTextFormatter.formattedString, "1234 56")
        XCTAssertEqual(inputTextFormatter.caretPosition, 7)
    }

    func testInserting() {
        guard let range = inputTextFormatter.formattedString.range(withOffset: 5, length: 0) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        inputTextFormatter.replaceCharacters(
            in: range,
            replacementString: "0000",
            emptySelection: true
        )

        XCTAssertEqual(inputTextFormatter.formattedString, "1234 0000 5678")
        XCTAssertEqual(inputTextFormatter.caretPosition, 9)
    }

    func testReplacingRange() {
        guard let range = inputTextFormatter.formattedString.range(withOffset: 5, length: 4) else {
            return XCTAssertNil("Out of range", file: #file, line: #line)
        }

        inputTextFormatter.replaceCharacters(
            in: range,
            replacementString: "0000",
            emptySelection: false
        )

        XCTAssertEqual(inputTextFormatter.formattedString, "1234 0000")
        XCTAssertEqual(inputTextFormatter.caretPosition, 9)
    }

    func testInvalidCharactersReplacesTextFieldTextWithFormattedString() {
        let invalidRange = NSRange(location: accountNumber.count + 1, length: 0)
        let textField = UITextField()

        _ = inputTextFormatter.textField(textField, shouldChangeCharactersIn: invalidRange, replacementString: "´")

        XCTAssertEqual(textField.text, inputTextFormatter.formattedString)
    }

    func testDeleteCharacterOutsideOfTokenBoundaryDoesNotDeleteAnything() {
        let invalidRange = NSRange(location: accountNumber.count + 1, length: 1)
        let textField = UITextField()

        _ = inputTextFormatter.textField(textField, shouldChangeCharactersIn: invalidRange, replacementString: "")

        XCTAssertEqual(textField.text, inputTextFormatter.formattedString)
    }

    func testDeleteLastCharacter() {
        let lastCharacterRange = NSRange(location: accountNumber.count, length: 1)
        let textField = UITextField()

        _ = inputTextFormatter.textField(textField, shouldChangeCharactersIn: lastCharacterRange, replacementString: "")

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
