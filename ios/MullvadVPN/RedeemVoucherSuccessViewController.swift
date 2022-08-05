//
//  RedeemVoucherSuccessViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-24.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class RedeemVoucherSuccessViewController: UIViewController {
    private let contentView: RedeemVoucherSuccessContentView

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    init(timeAdded: Int, newExpiry: String) {
        contentView = RedeemVoucherSuccessContentView(
            timeAdded: Self.formattedTimeAdded(from: timeAdded),
            paidUntil: Self.formattedNewExpiry(from: newExpiry)
        )
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        setUpContentView()
        setUpButtonTarget()
    }
}

private extension RedeemVoucherSuccessViewController {
    func setUpContentView() {
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    func setUpButtonTarget() {
        contentView.nextButton.addTarget(
            self,
            action: #selector(didTapNextButton),
            for: .touchUpInside
        )
    }

    @objc func didTapNextButton() {
        transitionToNextView()
    }

    func transitionToNextView() {
        var viewControllers = rootContainerController?.viewControllers ?? []
        viewControllers.removeFirstInstance(of: OutOfTimeViewController.self)
        viewControllers.removeFirstInstance(of: RedeemVoucherViewController.self)
        viewControllers.removeFirstInstance(of: RedeemVoucherSuccessViewController.self)

        rootContainerController?.setViewControllers(viewControllers, animated: true)
    }

    static func formattedTimeAdded(from timeAdded: Int) -> String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full
        formatter.calendar?.locale = .init(identifier: "us")

        return formatter.string(from: Double(timeAdded)) ?? ""
    }

    static func formattedNewExpiry(from newExpiry: String) -> String {
        let isoFormatter = DateFormatter(dateFormat: .iso8601)
        let formatter = DateFormatter(dateFormat: .standard)
        formatter.shortMonthSymbols = formatter.shortMonthSymbols
            .map { String($0.capitalized.prefix(3)) }
        guard let date = isoFormatter.date(from: newExpiry) else { return "" }

        return formatter.string(from: date)
    }
}
