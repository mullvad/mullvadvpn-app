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
     Pin edges to edges of other view edges.
     */
    func pinEdgesTo(
        _ other: UIView,
        insets: NSDirectionalEdgeInsets = .zero,
        excludingEdges: NSDirectionalRectEdge = []
    ) -> [NSLayoutConstraint] {
        var constraints = [NSLayoutConstraint]()

        if !excludingEdges.contains(.top) {
            constraints.append(
                topAnchor.constraint(equalTo: other.topAnchor, constant: insets.top)
            )
        }

        if !excludingEdges.contains(.bottom) {
            constraints.append(
                bottomAnchor.constraint(equalTo: other.bottomAnchor, constant: insets.bottom)
            )
        }

        if !excludingEdges.contains(.leading) {
            constraints.append(
                leadingAnchor.constraint(equalTo: other.leadingAnchor, constant: insets.leading)
            )
        }

        if !excludingEdges.contains(.trailing) {
            constraints.append(
                trailingAnchor.constraint(equalTo: other.trailingAnchor, constant: insets.trailing)
            )
        }

        return constraints
    }

    /**
     Pin edges to superview edges.
     */
    func pinEdgesToSuperview(
        insets: NSDirectionalEdgeInsets = .zero,
        excludingEdges: NSDirectionalRectEdge = []
    ) -> [NSLayoutConstraint] {
        guard let superview = superview else { return [] }

        return pinEdgesTo(superview, insets: insets, excludingEdges: excludingEdges)
    }

    /**
     Pin edges to superview margins.
     */
    func pinEdgesToSuperviewMargins(
        insets: NSDirectionalEdgeInsets = .zero,
        excludingEdges: NSDirectionalRectEdge = []
    ) -> [NSLayoutConstraint] {
        guard let superview = superview else { return [] }

        return pinEdgesToMargins(superview, insets: insets, excludingEdges: excludingEdges)
    }

    /**
     Pin edges to other view layout margins.
     */
    func pinEdgesToMargins(
        _ other: UIView,
        insets: NSDirectionalEdgeInsets = .zero,
        excludingEdges: NSDirectionalRectEdge = []
    ) -> [NSLayoutConstraint] {
        return pinEdgesTo(other.layoutMarginsGuide, insets: insets, excludingEdges: excludingEdges)
    }

    /**
     Pin edges to layout guide.
     */
    func pinEdgesTo(
        _ layoutGuide: UILayoutGuide,
        insets: NSDirectionalEdgeInsets = .zero,
        excludingEdges: NSDirectionalRectEdge = []
    ) -> [NSLayoutConstraint] {
        var constraints = [NSLayoutConstraint]()

        if !excludingEdges.contains(.top) {
            constraints.append(
                topAnchor.constraint(equalTo: layoutGuide.topAnchor, constant: insets.top)
            )
        }

        if !excludingEdges.contains(.bottom) {
            constraints.append(
                bottomAnchor.constraint(
                    equalTo: layoutGuide.bottomAnchor,
                    constant: insets.bottom
                )
            )
        }

        if !excludingEdges.contains(.leading) {
            constraints.append(
                leadingAnchor.constraint(
                    equalTo: layoutGuide.leadingAnchor,
                    constant: insets.leading
                )
            )
        }

        if !excludingEdges.contains(.trailing) {
            constraints.append(
                trailingAnchor.constraint(
                    equalTo: layoutGuide.trailingAnchor,
                    constant: insets.trailing
                )
            )
        }

        return constraints
    }

    /**
     Pin horizontal edges to other view edges.
     */
    func pinHorizontalEdgesTo(
        _ other: UIView,
        leadingInset: CGFloat = .zero,
        trailingInset: CGFloat = .zero
    ) -> [NSLayoutConstraint] {
        return pinEdgesTo(
            other,
            insets: NSDirectionalEdgeInsets(
                top: 0,
                leading: leadingInset,
                bottom: 0,
                trailing: trailingInset
            ),
            excludingEdges: [.bottom, .top]
        )
    }

    /**
     Pin horizontal edges to other view layout margins.
     */
    func pinHorizontalEdgesToMargins(
        _ other: UIView,
        leadingInset: CGFloat = .zero,
        trailingInset: CGFloat = .zero
    ) -> [NSLayoutConstraint] {
        return pinEdgesToMargins(
            other,
            insets: NSDirectionalEdgeInsets(
                top: 0,
                leading: leadingInset,
                bottom: 0,
                trailing: trailingInset
            ),
            excludingEdges: [.bottom, .top]
        )
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
    subview.pinEdgesToSuperview(
        insets: NSDirectionalEdgeInsets(top: 8, leading: 16, bottom: 8, trailing: 24),
        excludingEdges: .bottom
    )
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
