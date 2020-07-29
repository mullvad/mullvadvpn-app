//
//  DisconnectSplitButton.swift
//  MullvadVPN
//
//  Created by pronebird on 29/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

private let kSplitSeparatorWidth = CGFloat(1)

class DisconnectSplitButton: UIView {
    @IBOutlet var contentView: UIView!
    @IBOutlet var primaryButton: AppButton!
    @IBOutlet var secondaryButton: AppButton!

    private var secondaryButtonObserver: NSObjectProtocol?

    init(bundle: Bundle?) {
        super.init(frame: .zero)

        loadFromNib(bundle: bundle)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func loadFromNib(bundle: Bundle?) {
        let nib = UINib(nibName: "DisconnectSplitButton", bundle: bundle)
        _ = nib.instantiate(withOwner: self, options: nil)

        primaryButton.titleLabel?.font = UIFont.systemFont(ofSize: 18, weight: .semibold)

        contentView.translatesAutoresizingMaskIntoConstraints = false
        addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.leadingAnchor.constraint(equalTo: leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: trailingAnchor),
            contentView.topAnchor.constraint(equalTo: topAnchor),
            contentView.bottomAnchor.constraint(equalTo: bottomAnchor)
        ])

        secondaryButtonObserver = secondaryButton.observe(\.bounds, options: [.new]) { [weak self] (button, change) in
            self?.adjustTitleLabelPosition()
        }
    }

    private func adjustTitleLabelPosition() {
        var contentInsets = AppButton.defaultContentInsets
        contentInsets.left = secondaryButton.frame.width + kSplitSeparatorWidth
        contentInsets.right = 0

        primaryButton.contentEdgeInsets = contentInsets
    }
}
