//
//  MarkdownParser.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Markdown grammar.
private enum MarkdownToken {
    static let asterisk: Character = "*"
    static let openBrace: Character = "["
    static let closeBrace: Character = "]"
    static let openParen: Character = "("
    static let closeParen: Character = ")"
}

/**
 A simple markdown parser.

 The following markdown syntax is currently supported:

 1. Bold text: `**bold text**`
 2. Links: `[Mullvad VPN](https://mullvad.net)`
 3. Paragraphs represented by two line separators between text.
 4. Plain unstyled text nodes.
 */
@available(iOS, introduced: 14.0, obsoleted: 15.0, message: "Replace with native support for Markdown.")
struct MarkdownParser {
    private var iterator: PeekableIterator<String.Iterator>
    private let documentNode = MarkdownDocumentNode()

    /// Initializes the parser with a markdown string.
    /// - Parameter markdown: markdown string.
    init(markdown: String) {
        iterator = PeekableIterator(markdown.makeIterator())
    }

    /// Parse markdown into the tree structure.
    ///
    /// - Returns: a document node.
    mutating func parse() -> MarkdownDocumentNode {
        while let char = iterator.next() {
            // Parse bold tag **text**
            if char == MarkdownToken.asterisk, char == iterator.peek() {
                _ = iterator.next() // consume peeked element

                // If current node is bold then we found a closing tag for it.
                if let boldNode = documentNode.lastChild as? MarkdownBoldNode, !boldNode.isClosed {
                    boldNode.markClosed()
                } else {
                    documentNode.addChild(MarkdownBoldNode(isClosed: false))
                }

                continue
            }

            // Parse URL [title](url)
            if char == MarkdownToken.openBrace, let linkNode = try? tryParseLink() {
                documentNode.addChild(linkNode)
                continue
            }

            // Parse paragraphs separated by a sequence of two CRLF.
            // Swift string iterator parses CRLF into a single character.
            if char.isNewline, let nextChar = iterator.peek(), nextChar.isNewline {
                _ = iterator.next() // consume peeked element
                wrapIntoParagraph()
                continue
            }

            // Found untagged text.
            switch documentNode.lastChild?.type {
            case .bold:
                if let boldNode = documentNode.lastChild as? MarkdownBoldNode, !boldNode.isClosed {
                    boldNode.appendText(String(char))
                } else {
                    let textNode = MarkdownTextNode(text: String(char))

                    documentNode.addChild(textNode)
                }

            case .text:
                let textNode = documentNode.lastChild as? MarkdownTextNode
                textNode?.appendText(String(char))

            case .link, .paragraph, .document, .none:
                let textNode = MarkdownTextNode(text: String(char))

                documentNode.addChild(textNode)
            }
        }

        // Wrap the remaining nodes into paragraph.
        wrapIntoParagraph()

        return documentNode
    }

    /// Wraps all preceding nodes into a paragraph traversing in reverse until either the beginning of the document is reached or another paragraph.
    private func wrapIntoParagraph() {
        var extractedChildren = [MarkdownNode]()

        for child in documentNode.children.reversed() {
            guard child.type != .paragraph else { break }

            child.removeFromParent()
            extractedChildren.insert(child, at: 0)
        }

        guard !extractedChildren.isEmpty else { return }

        let paragraph = MarkdownParagraphNode()
        extractedChildren.forEach { paragraph.addChild($0) }
        documentNode.addChild(paragraph)
    }

    /// Parse markdown link.
    ///
    /// Advances the cursor of internal iterator upon success.
    ///
    /// Markdown links have the following syntax: `[Mullvad VPN](https://mullvad.net)`.
    /// The copy of an iterator has already consumed the first `[` letter.
    ///
    /// - Returns: an instance of `MarkdownLinkNode` upon success, otherwise throws an error.
    private mutating func tryParseLink() throws -> MarkdownLinkNode {
        // Copy the underlying iterator to prevent advancing the cursor in case of failure to parse the link.
        var tempIterator = iterator

        // Parse the title. `[` is already consumed by the caller.
        let title = try tempIterator.take { ch in
            guard !ch.isNewline else { throw MarkdownParseURLError() }
            return ch != MarkdownToken.closeBrace
        }

        // Parse the opening paren.
        guard tempIterator.next() == MarkdownToken.openParen else { throw MarkdownParseURLError() }

        // Parse URL until the closing paren.
        var isFoundClosingParen = false
        let url = try tempIterator.take { ch in
            guard !ch.isNewline else { throw MarkdownParseURLError() }
            isFoundClosingParen = ch == MarkdownToken.closeParen
            return !isFoundClosingParen
        }
        guard isFoundClosingParen else { throw MarkdownParseURLError() }

        // Replace the underlying iterator to advance the cursor.
        iterator = tempIterator

        return MarkdownLinkNode(title: title, url: url)
    }
}

/// Internal error type used to indicate URL parsing error.
private struct MarkdownParseURLError: Error {}
