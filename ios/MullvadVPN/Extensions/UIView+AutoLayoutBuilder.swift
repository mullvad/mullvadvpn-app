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
        return edges.inner.map { edge -> NSLayoutConstraint in
            switch edge {
            case let .top(inset):
                return topAnchor.constraint(equalTo: other.topAnchor, constant: inset)

            case let .bottom(inset):
                return bottomAnchor.constraint(equalTo: other.bottomAnchor, constant: inset)

            case let .leading(inset):
                return leadingAnchor.constraint(equalTo: other.leadingAnchor, constant: inset)

            case let .trailing(inset):
                return trailingAnchor.constraint(equalTo: other.trailingAnchor, constant: inset)
            }
        }
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
        return edges.inner.map { edge -> NSLayoutConstraint in
            switch edge {
            case let .top(inset):
                return topAnchor.constraint(equalTo: layoutGuide.topAnchor, constant: inset)

            case let .bottom(inset):
                return bottomAnchor.constraint(equalTo: layoutGuide.bottomAnchor, constant: inset)

            case let .leading(inset):
                return leadingAnchor.constraint(equalTo: layoutGuide.leadingAnchor, constant: inset)

            case let .trailing(inset):
                return trailingAnchor.constraint(
                    equalTo: layoutGuide.trailingAnchor,
                    constant: inset
                )
            }
        }
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

extension UIView {
    /**
     Add subviews using AutoLayout and configure constraints.
     */
    func addConstrainedSubviews(
        _ subviews: [UIView],
        @AutoLayoutBuilder builder: () -> [NSLayoutConstraint]
    ) {
        for subview in subviews {
            subview.configureForAutoLayout()
            addSubview(subview)
        }

        NSLayoutConstraint.activate(builder())
    }

    /**
     Add subviews using AutoLayout without configuring constraints.
     */
    func addConstrainedSubviews(_ subviews: [UIView]) {
        addConstrainedSubviews(subviews) {}
    }

    /**
     Configure view for AutoLayout by disabling automatic autoresizing mask translation into
     constraints.
     */
    func configureForAutoLayout() {
        translatesAutoresizingMaskIntoConstraints = false
    }
}

/**
 Struct describing a relationship between AutoLayout anchors.
 */
struct PinnableEdges {
    /**
     Enum describing each inidividual edge with associated inset value.
     */
    enum Edge: Hashable {
        case top(CGFloat)
        case bottom(CGFloat)
        case leading(CGFloat)
        case trailing(CGFloat)

        var rectEdge: NSDirectionalRectEdge {
            switch self {
            case .top:
                return .top
            case .bottom:
                return .bottom
            case .leading:
                return .leading
            case .trailing:
                return .trailing
            }
        }

        func hash(into hasher: inout Hasher) {
            hasher.combine(rectEdge.rawValue)
        }

        static func == (lhs: Self, rhs: Self) -> Bool {
            return lhs.rectEdge == rhs.rectEdge
        }
    }

    /**
     Inner set of `Edge` objects.
     */
    var inner: Set<Edge>

    /**
     Designated initializer.
     */
    init(_ edges: Set<Edge>) {
        inner = edges
    }

    /**
     Returns new `PinnableEdges` with the given edge(s) excluded.
     */
    func excluding(_ excludeEdges: NSDirectionalRectEdge) -> Self {
        return Self(inner.filter { !excludeEdges.contains($0.rectEdge) })
    }

    /**
     Returns new `PinnableEdges` initialized with four edges and corresponding insets from
     `NSDirectionalEdgeInsets`.
     */
    static func all(_ directionalEdgeInsets: NSDirectionalEdgeInsets = .zero) -> Self {
        return Self([
            .top(directionalEdgeInsets.top),
            .bottom(directionalEdgeInsets.bottom),
            .leading(directionalEdgeInsets.leading),
            .trailing(directionalEdgeInsets.trailing),
        ])
    }
}
