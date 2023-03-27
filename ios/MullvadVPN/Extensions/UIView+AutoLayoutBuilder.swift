//
//  UIView+AutoLayoutBuilder.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIView {
    /**
     Pin all edges to edges of other view.
     */
    func pinEdgesTo(_ other: UIView) -> [NSLayoutConstraint] {
        return pinEdges(.all(), to: other)
    }

    /**
     Pin edges to edges of other view.
     */
    func pinEdges(_ edges: PinnableEdges, to other: UIView) -> [NSLayoutConstraint] {
        var constraints = [NSLayoutConstraint]()

        if edges.contains(.top) {
            constraints.append(
                topAnchor.constraint(equalTo: other.topAnchor, constant: edges.top)
            )
        }

        if edges.contains(.bottom) {
            constraints.append(
                bottomAnchor.constraint(equalTo: other.bottomAnchor, constant: edges.bottom)
            )
        }

        if edges.contains(.leading) {
            constraints.append(
                leadingAnchor.constraint(equalTo: other.leadingAnchor, constant: edges.leading)
            )
        }

        if edges.contains(.trailing) {
            constraints.append(
                trailingAnchor.constraint(equalTo: other.trailingAnchor, constant: edges.trailing)
            )
        }

        return constraints
    }

    /**
     Pin edges to superview edges.
     */
    func pinEdgesToSuperview(_ edges: PinnableEdges = .all()) -> [NSLayoutConstraint] {
        guard let superview = superview else { return [] }

        return pinEdges(edges, to: superview)
    }

    /**
     Pin edges to superview margins.
     */
    func pinEdgesToSuperviewMargins(_ edges: PinnableEdges = .all()) -> [NSLayoutConstraint] {
        guard let superview = superview else { return [] }

        return pinEdges(edges, toMarginsOf: superview)
    }

    /**
     Pin all edges to other view layout margins.
     */
    func pinEdgesToMarginsOf(_ other: UIView) -> [NSLayoutConstraint] {
        return pinEdges(.all(), toMarginsOf: other)
    }

    /**
     Pin edges to other view layout margins.
     */
    func pinEdges(_ edges: PinnableEdges, toMarginsOf other: UIView) -> [NSLayoutConstraint] {
        return pinEdges(edges, to: other.layoutMarginsGuide)
    }

    /**
     Pin edges to layout guide.
     */
    func pinEdges(_ edges: PinnableEdges, to layoutGuide: UILayoutGuide) -> [NSLayoutConstraint] {
        var constraints = [NSLayoutConstraint]()

        if edges.contains(.top) {
            constraints.append(
                topAnchor.constraint(equalTo: layoutGuide.topAnchor, constant: edges.top)
            )
        }

        if edges.contains(.bottom) {
            constraints.append(
                bottomAnchor.constraint(
                    equalTo: layoutGuide.bottomAnchor,
                    constant: edges.bottom
                )
            )
        }

        if edges.contains(.leading) {
            constraints.append(
                leadingAnchor.constraint(
                    equalTo: layoutGuide.leadingAnchor,
                    constant: edges.leading
                )
            )
        }

        if edges.contains(.trailing) {
            constraints.append(
                trailingAnchor.constraint(
                    equalTo: layoutGuide.trailingAnchor,
                    constant: edges.trailing
                )
            )
        }

        return constraints
    }
}

/**
 AutoLayout builder.

 Use it in conjunction with `NSLayoutConstraint.activate()`, for example:

 ```
 let view = UIView()
 let subview = UIView()

 subview.translatesAutoresizingMaskIntoConstraints = false

 view.addSubview(subview)

 NSLayoutConstraint.activate {
    // Pin top, leading and trailing edges to superview with associated margins.
    subview.pinEdgesToSuperview(.top(8).leading(16).trailing(24))

    // Pin bottom to safe area layout guide.
    subview.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor)
 }

 ```
 */
@resultBuilder enum AutoLayoutBuilder {
    static func buildBlock(_ components: [NSLayoutConstraint]...) -> [NSLayoutConstraint] {
        return components.flatMap { $0 }
    }

    static func buildExpression(_ expression: NSLayoutConstraint) -> [NSLayoutConstraint] {
        return [expression]
    }

    static func buildExpression(_ expression: [NSLayoutConstraint]) -> [NSLayoutConstraint] {
        return expression
    }

    static func buildOptional(_ components: [NSLayoutConstraint]?) -> [NSLayoutConstraint] {
        return components ?? []
    }

    static func buildEither(first components: [NSLayoutConstraint]) -> [NSLayoutConstraint] {
        return components
    }

    static func buildEither(second components: [NSLayoutConstraint]) -> [NSLayoutConstraint] {
        return components
    }

    static func buildArray(_ components: [[NSLayoutConstraint]]) -> [NSLayoutConstraint] {
        return components.flatMap { $0 }
    }
}

extension NSLayoutConstraint {
    /**
     Activate constraints produced by a builder.
     */
    static func activate(@AutoLayoutBuilder builder: () -> [NSLayoutConstraint]) {
        activate(builder())
    }
}

/**
 Struct describing which edges to pin when creating AutoLayout constraints.
 */
struct PinnableEdges {
    /// Edges that will be pinned.
    private var rectEdge: NSDirectionalRectEdge = []

    /// Container used for hoding edge insets.
    private var insets: NSDirectionalEdgeInsets = .zero

    /// Returns edges configured to pin to the top anchor.
    static func top(_ inset: CGFloat = 0) -> Self {
        return PinnableEdges().top(inset)
    }

    /// Returns edges configured to pin to leading anchor.
    static func leading(_ inset: CGFloat = 0) -> Self {
        return PinnableEdges().leading(inset)
    }

    /// Returns edges configured to pin to trailing anchor.
    static func trailing(_ inset: CGFloat = 0) -> Self {
        return PinnableEdges().trailing(inset)
    }

    /// Returns edges configured to pin to bottom anchor.
    static func bottom(_ inset: CGFloat = 0) -> Self {
        return PinnableEdges().bottom(inset)
    }

    /// Returns edges configured to pin to horizontal (leading and trailing) anchors.
    static func horizontal(_ inset: CGFloat = 0) -> Self {
        return PinnableEdges().horizontal(inset)
    }

    /// Returns edges configured to pin to vertical (top and bottom) anchors.
    static func vertical(_ inset: CGFloat = 0) -> Self {
        return PinnableEdges().vertical(inset)
    }

    /// Returns edges configured to pin to all anchors, optionally accepting insets for all edges.
    static func all(_ directionalEdgeInsets: NSDirectionalEdgeInsets = .zero) -> PinnableEdges {
        return PinnableEdges(rectEdge: .all, insets: directionalEdgeInsets)
    }

    /// Returns `true` if the struct is configured to pin the given edge.
    func contains(_ edge: NSDirectionalRectEdge) -> Bool {
        return rectEdge.contains(edge)
    }

    /// Top edge inset.
    var top: CGFloat {
        return insets.top
    }

    /// Leading edge inset.
    var leading: CGFloat {
        return insets.leading
    }

    /// Trailing edge inset.
    var trailing: CGFloat {
        return insets.trailing
    }

    /// Bottom edge inset.
    var bottom: CGFloat {
        return insets.bottom
    }

    /// Returns a copy of struct with the given edges excluded and corresponding insets being
    /// zeroed.
    func excluding(_ edges: NSDirectionalRectEdge) -> Self {
        var newEdges = self

        if edges.contains(.top) {
            newEdges.insets.top = 0
        }

        if edges.contains(.bottom) {
            newEdges.insets.bottom = 0
        }

        if edges.contains(.leading) {
            newEdges.insets.leading = 0
        }

        if edges.contains(.trailing) {
            newEdges.insets.trailing = 0
        }

        newEdges.rectEdge.remove(edges)

        return newEdges
    }

    /// Returns edges configured to pin to horizontal (leading and trailing) anchors.
    func horizontal(_ inset: CGFloat = 0) -> Self {
        var newEdges = self
        newEdges.insets.leading = inset
        newEdges.insets.trailing = inset
        newEdges.rectEdge.formUnion([.leading, .trailing])
        return newEdges
    }

    /// Returns edges configured to pin to vertical (top and bottom) anchors.
    func vertical(_ inset: CGFloat = 0) -> Self {
        var newEdges = self
        newEdges.insets.top = inset
        newEdges.insets.bottom = inset
        newEdges.rectEdge.formUnion([.top, .bottom])
        return newEdges
    }

    /// Returns edges configured to pin to the top anchor.
    func top(_ inset: CGFloat = 0) -> Self {
        var newEdges = self
        newEdges.insets.top = inset
        newEdges.rectEdge.insert(.top)
        return newEdges
    }

    /// Returns edges configured to pin to leading anchor.
    func leading(_ inset: CGFloat = 0) -> Self {
        var newEdges = self
        newEdges.insets.leading = inset
        newEdges.rectEdge.insert(.leading)
        return newEdges
    }

    /// Returns edges configured to pin to trailing anchor.
    func trailing(_ inset: CGFloat = 0) -> Self {
        var newEdges = self
        newEdges.insets.trailing = inset
        newEdges.rectEdge.insert(.trailing)
        return newEdges
    }

    /// Returns edges configured to pin to bottom anchor.
    func bottom(_ inset: CGFloat = 0) -> Self {
        var newEdges = self
        newEdges.insets.bottom = inset
        newEdges.rectEdge.insert(.bottom)
        return newEdges
    }
}
