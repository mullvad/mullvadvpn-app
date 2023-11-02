//
//  UITableView+ReuseIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**
 Type describing cell identifier.

 Example:

 ```
 enum MyCellIdentifier: CaseIterable, String, CellIdentifierProtocol {
    case primary, secondary

    var cellClass: AnyClass {
        switch self {
            case .primary:
                return MyPrimaryCell.self
            case .secondary:
                return MySecondaryCell.self
        }
    }
 }

 // Register all cells
 tableView.registerReusableViews(from: MyCellIdentifier.self)

 // Dequeue cell
 let cell = tableView.dequeueReusableView(withIdentifier: MyCellIdentifier.primary, for: IndexPath(row: 0, section: 0)))
 ```
 */
protocol CellIdentifierProtocol: CaseIterable, RawRepresentable where RawValue == String {
    var cellClass: AnyClass { get }
}

/**
 Type describing header footer view identifier.

 Example:

 ```
 enum MyHeaderFooterIdentifier: CaseIterable, String, HeaderFooterIdentifierProtocol {
    case primary, secondary

    var headerFooterClass: AnyClass {
        switch self {
            case .primary:
                return MyPrimaryHeaderFooterView.self
            case .secondary:
                return MySecondaryHeaderFooterView.self
        }
    }
 }

 // Register all cells
 tableView.registerReusableViews(from: MyHeaderFooterIdentifier.self)

 // Dequeue header footer view
 let headerFooterView = tableView.dequeueReusableView(withIdentifier: MyHeaderFooterIdentifier.primary)
 ```
 */
protocol HeaderFooterIdentifierProtocol: CaseIterable, RawRepresentable where RawValue == String {
    var headerFooterClass: AnyClass { get }
}

extension UITableView {
    /// Register all cell identifiers in the table view.
    /// - Parameter cellIdentifierType: a type conforming to the ``CellIdentifierProtocol`` protocol.
    func registerReusableViews<T: CellIdentifierProtocol>(from cellIdentifierType: T.Type) {
        cellIdentifierType.allCases.forEach { identifier in
            register(identifier.cellClass, forCellReuseIdentifier: identifier.rawValue)
        }
    }

    /// Register header footer view identifiers in the table view.
    /// - Parameter headerFooterIdentifierType: a type conforming to the ``HeaderFooterIdentifierProtocol`` protocol.
    func registerReusableViews<T: HeaderFooterIdentifierProtocol>(from headerFooterIdentifierType: T.Type) {
        headerFooterIdentifierType.allCases.forEach { identifier in
            register(identifier.headerFooterClass, forHeaderFooterViewReuseIdentifier: identifier.rawValue)
        }
    }
}

extension UITableView {
    /// Convenience method to dequeue a cell by identifier conforming to ``CellIdentifierProtocol``.
    /// - Parameters:
    ///   - identifier: cell identifier.
    ///   - indexPath: index path
    /// - Returns: table cell.
    func dequeueReusableView(
        withIdentifier identifier: some CellIdentifierProtocol,
        for indexPath: IndexPath
    ) -> UITableViewCell {
        dequeueReusableCell(withIdentifier: identifier.rawValue, for: indexPath)
    }

    /// Convenience method to dequeue a header footer view by identifier conforming to ``HeaderFooterIdentifierProtocol``.
    /// - Parameter identifier: header footer view identifier.
    /// - Returns: table header footer view.
    func dequeueReusableView(withIdentifier identifier: some HeaderFooterIdentifierProtocol)
        -> UITableViewHeaderFooterView? {
        dequeueReusableHeaderFooterView(withIdentifier: identifier.rawValue)
    }
}
