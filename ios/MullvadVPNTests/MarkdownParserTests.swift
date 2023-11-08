//
//  MarkdownParserTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 07/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import XCTest

final class MarkdownParserTests: XCTestCase {
    let defaultStylingOptions = MarkdownStylingOptions(
        font: UIFont(name: "Courier", size: 9)!,
        linkColor: UIColor.red
    )

    func testParsingText() {
        var parser = MarkdownParser(markdown: "Untagged text")
        let document = parser.parse()

        XCTAssertEqual(document, MarkdownDocumentNode(children: [
            MarkdownParagraphNode(children: [
                MarkdownTextNode(text: "Untagged text"),
            ]),
        ]))
    }

    func testParsingBold() {
        var parser = MarkdownParser(markdown: "**bold text**")
        let document = parser.parse()

        XCTAssertEqual(document, MarkdownDocumentNode(children: [
            MarkdownParagraphNode(children: [
                MarkdownBoldNode(text: "bold text"),
            ]),
        ]))
    }

    func testParsingUnclosedBold() {
        var parser = MarkdownParser(markdown: "**bold text")
        let document = parser.parse()

        XCTAssertEqual(document, MarkdownDocumentNode(children: [
            MarkdownParagraphNode(children: [
                MarkdownBoldNode(text: "bold text"),
            ]),
        ]))
    }

    func testParsingLinks() {
        var parser = MarkdownParser(markdown: "[Mullvad VPN](https://mullvad.net/)")
        let document = parser.parse()

        XCTAssertEqual(document, MarkdownDocumentNode(children: [
            MarkdownParagraphNode(children: [
                MarkdownLinkNode(title: "Mullvad VPN", url: "https://mullvad.net/"),
            ]),
        ]))
    }

    func testParsingMalformedLinks() {
        var parser = MarkdownParser(markdown: "[Mullvad VPN](https://mullvad.net/")
        let document = parser.parse()

        XCTAssertEqual(document, MarkdownDocumentNode(children: [
            MarkdownParagraphNode(children: [
                MarkdownTextNode(text: "[Mullvad VPN](https://mullvad.net/"),
            ]),
        ]))
    }

    func testParsingParagraphs() {
        var parser = MarkdownParser(markdown: "Paragraph 1\nStill paragraph 1\n\nParagraph 2")
        let document = parser.parse()

        XCTAssertEqual(document, MarkdownDocumentNode(children: [
            MarkdownParagraphNode(children: [
                MarkdownTextNode(text: "Paragraph 1\nStill paragraph 1"),
            ]),
            MarkdownParagraphNode(children: [
                MarkdownTextNode(text: "Paragraph 2"),
            ]),
        ]))
    }

    func testTransformingBoldToAttributedString() {
        var parser = MarkdownParser(markdown: "**bold text**")
        let attributedString = parser.parse().attributedString(options: defaultStylingOptions)

        let expectedString = NSMutableAttributedString()
        paragraph(appendingInto: expectedString) {
            bold("bold text")
        }

        XCTAssertTrue(expectedString.isEqual(to: attributedString))
    }

    func testTransformingLinkToAttributedString() {
        var parser = MarkdownParser(markdown: "[Mullvad VPN](https://mullvad.net/)")
        let attributedString = parser.parse().attributedString(options: defaultStylingOptions)

        let expectedString = NSMutableAttributedString()
        paragraph(appendingInto: expectedString) {
            link(title: "Mullvad VPN", url: "https://mullvad.net/")
        }

        XCTAssertTrue(expectedString.isEqual(to: attributedString))
    }

    func testTransformingParagraphsToAttributedString() {
        var parser = MarkdownParser(markdown: "Paragraph 1\nStill paragraph 1\n\nParagraph 2")
        let parsedString = parser.parse().attributedString(options: defaultStylingOptions)

        let expectedString = NSMutableAttributedString()
        paragraph(appendingInto: expectedString) {
            text("Paragraph 1\u{2028}Still paragraph 1\n")
        }
        paragraph(appendingInto: expectedString) {
            text("Paragraph 2")
        }

        XCTAssertTrue(parsedString.isEqual(to: expectedString))
    }

    func testTransformingComplexMarkdownToAttributedString() {
        let markdown = """
        Manage default and setup custom methods to access to Mullvad VPN API. [About API access...](#about)

        **Important:** direct access method **cannot** be removed.
        """

        var parser = MarkdownParser(markdown: markdown)
        let parsedString = parser.parse().attributedString(options: defaultStylingOptions)

        let expectedString = NSMutableAttributedString()

        paragraph(appendingInto: expectedString) {
            text("Manage default and setup custom methods to access to Mullvad VPN API. ")
            link(title: "About API access...", url: "#about")
            text("\n")
        }
        paragraph(appendingInto: expectedString) {
            bold("Important:")
            text(" direct access method ")
            bold("cannot")
            text(" be removed.")
        }
        XCTAssertTrue(parsedString.isEqual(to: expectedString))
    }
}

private extension MarkdownParserTests {
    func paragraph(
        appendingInto resultString: NSMutableAttributedString,
        @ParagraphBuilder builder: () -> [NSAttributedString]
    ) {
        let mutableString = NSMutableAttributedString()

        builder().forEach { mutableString.append($0) }
        mutableString.addAttribute(
            .paragraphStyle,
            value: defaultStylingOptions.paragraphStyle,
            range: NSRange(location: 0, length: mutableString.length)
        )

        resultString.append(mutableString)
    }

    func link(title: String, url: String) -> NSAttributedString {
        var attributes: [NSAttributedString.Key: Any] = [
            .font: defaultStylingOptions.font,
            .link: url,
        ]
        if let linkColor = defaultStylingOptions.linkColor {
            attributes[.foregroundColor] = linkColor
        }
        return NSAttributedString(string: title, attributes: attributes)
    }

    func bold(_ text: String) -> NSAttributedString {
        NSAttributedString(string: text, attributes: [.font: defaultStylingOptions.boldFont])
    }

    func text(_ text: String) -> NSAttributedString {
        NSAttributedString(string: text, attributes: [.font: defaultStylingOptions.font])
    }
}

@resultBuilder
private enum ParagraphBuilder {
    static func buildPartialBlock(first: NSAttributedString) -> [NSAttributedString] { [first] }
    static func buildPartialBlock(first: [NSAttributedString]) -> [NSAttributedString] { first }

    static func buildPartialBlock(
        accumulated: [NSAttributedString],
        next: NSAttributedString
    ) -> [NSAttributedString] {
        accumulated + [next]
    }

    static func buildPartialBlock(
        accumulated: [NSAttributedString],
        next: [NSAttributedString]
    ) -> [NSAttributedString] {
        accumulated + next
    }

    static func buildBlock() -> [NSAttributedString] { [] }
}
