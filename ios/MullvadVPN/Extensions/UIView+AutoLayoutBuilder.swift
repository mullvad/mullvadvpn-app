//
//  UIView+AutoLayoutBuilder.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**
 Protocol that describes common AutoLayout properties of `UIView` and `UILayoutGuide` and helps to remove distinction
 between two of them when creating constraints.
 */
protocol AutoLayoutAnchorsProtocol {
    var topAnchor: NSLayoutYAxisAnchor { get }
    var bottomAnchor: NSLayoutYAxisAnchor { get }
    var leadingAnchor: NSLayoutXAxisAnchor { get }
    var trailingAnchor: NSLayoutXAxisAnchor { get }
}

extension UIView: AutoLayoutAnchorsProtocol {}
extension UILayoutGuide: AutoLayoutAnchorsProtocol {}

extension UIView {
    /**
     Pin all edges to edges of other view.
     */
    func pinEdgesTo(_ other: AutoLayoutAnchorsProtocol) -> [NSLayoutConstraint] {
        pinEdges(.all(), to: other)
    }

    /**
     Pin edges to edges of other view.
     */
    func pinEdges(_ edges: PinnableEdges, to other: AutoLayoutAnchorsProtocol) -> [NSLayoutConstraint] {
        edges.makeConstraints(firstView: self, secondView: other)
    }

    /**
     Pin edges to superview edges.
     */
    func pinEdgesToSuperview(_ edges: PinnableEdges = .all()) -> [NSLayoutConstraint] {
        guard let superview else { return [] }

        return pinEdges(edges, to: superview)
    }

    /**
     Pin edges to superview margins.
     */
    func pinEdgesToSuperviewMargins(_ edges: PinnableEdges = .all()) -> [NSLayoutConstraint] {
        guard let superview else { return [] }

        return pinEdges(edges, to: superview.layoutMarginsGuide)
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
    // Pin top, leading and trailing edges to superview.
    subview.pinEdgesToSuperview(.init([.top(8), .leading(16), .trailing(8)]))

    // Pin bottom to safe area layout guide.
    subview.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor)
 }
 ```
 */
@resultBuilder enum AutoLayoutBuilder {
    static func buildBlock(_ components: [NSLayoutConstraint]...) -> [NSLayoutConstraint] {
        components.flatMap { $0 }
    }

    static func buildExpression(_ expression: NSLayoutConstraint) -> [NSLayoutConstraint] {
        [expression]
    }

    static func buildExpression(_ expression: [NSLayoutConstraint]) -> [NSLayoutConstraint] {
        expression
    }

    static func buildOptional(_ components: [NSLayoutConstraint]?) -> [NSLayoutConstraint] {
        components ?? []
    }

    static func buildEither(first components: [NSLayoutConstraint]) -> [NSLayoutConstraint] {
        components
    }

    static func buildEither(second components: [NSLayoutConstraint]) -> [NSLayoutConstraint] {
        components
    }

    static func buildArray(_ components: [[NSLayoutConstraint]]) -> [NSLayoutConstraint] {
        components.flatMap { $0 }
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
            lhs.rectEdge == rhs.rectEdge
        }

        func makeConstraint(
            firstView: AutoLayoutAnchorsProtocol,
            secondView: AutoLayoutAnchorsProtocol
        ) -> NSLayoutConstraint {
            switch self {
            case let .top(inset):
                return firstView.topAnchor.constraint(equalTo: secondView.topAnchor, constant: inset)

            case let .bottom(inset):
                return firstView.bottomAnchor.constraint(equalTo: secondView.bottomAnchor, constant: -inset)

            case let .leading(inset):
                return firstView.leadingAnchor.constraint(equalTo: secondView.leadingAnchor, constant: inset)

            case let .trailing(inset):
                return firstView.trailingAnchor.constraint(equalTo: secondView.trailingAnchor, constant: -inset)
            }
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
        Self(inner.filter { !excludeEdges.contains($0.rectEdge) })
    }

    /**
     Returns new `PinnableEdges` initialized with four edges and corresponding insets from
     `NSDirectionalEdgeInsets`.
     */
    static func all(_ directionalEdgeInsets: NSDirectionalEdgeInsets = .zero) -> Self {
        Self([
            .top(directionalEdgeInsets.top),
            .bottom(directionalEdgeInsets.bottom),
            .leading(directionalEdgeInsets.leading),
            .trailing(directionalEdgeInsets.trailing),
        ])
    }

    /**
     Returns new constraints pinning edges of the corresponding views.
     */
    func makeConstraints(
        firstView: AutoLayoutAnchorsProtocol,
        secondView: AutoLayoutAnchorsProtocol
    ) -> [NSLayoutConstraint] {
        inner.map { $0.makeConstraint(firstView: firstView, secondView: secondView) }
    }
}
