//
//  MarkdownNode.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// The base type defining markdown node.
/// Do not instantiate this type directly. Use one of its subclasses instead.
class MarkdownNode: CustomDebugStringConvertible {
    /// The type of node.
    let type: MarkdownNodeType

    /// The child nodes.
    private(set) var children: [MarkdownNode] = []

    /// The parent node.
    private(set) weak var parent: MarkdownNode?

    init(type: MarkdownNodeType, children: [MarkdownNode] = []) {
        self.type = type
        children.forEach { addChild($0) }
    }

    /// Returns last child.
    var lastChild: MarkdownNode? {
        return children.last
    }

    /// Add child node.
    func addChild(_ child: MarkdownNode) {
        child.parent = self
        children.append(child)
    }

    /// Remove child.
    func removeChild(_ child: MarkdownNode) {
        children.removeAll { childFromArray in
            guard child === childFromArray else { return false }

            child.parent = nil
            return true
        }
    }

    /// Detach this node from parent.
    func removeFromParent() {
        parent?.removeChild(self)
    }

    var debugDescription: String {
        // Subclasses should override this method.
        return "\(self)"
    }

    /// Returns a recursive description of a markdown subtree. Useful when debugging.
    ///
    /// - Parameter level: indentation level.
    /// - Returns: recursive description of a subtree
    func recursiveDescription(level: Int = 0) -> String {
        let indent = String(repeating: "  ", count: level)
        var str = ""

        let descriptionLines = debugDescription.components(separatedBy: .newlines)
        if let firstLine = descriptionLines.first {
            str += "\(indent)+ \(firstLine)"
        }
        descriptionLines.dropFirst().forEach { line in
            str += "\n\(indent)  \(line)"
        }

        for child in children {
            str += "\n" + child.recursiveDescription(level: level + 1)
        }

        return str
    }

    /// Test equality.
    ///
    /// Default implementation only checks node types.
    ///
    /// - Parameter other: other node.
    /// - Returns: `true` if objects are equal, otherwise `false`.
    func isEqualTo(_ other: MarkdownNode) -> Bool {
        guard type == other.type && children.count == other.children.count else { return false }

        return zip(children, other.children).allSatisfy { $0.isEqualTo($1) }
    }
}

extension MarkdownNode: Equatable {
    static func == (lhs: MarkdownNode, rhs: MarkdownNode) -> Bool {
        return lhs.isEqualTo(rhs)
    }
}
