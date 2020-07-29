//
//  DisconnectSplitButton.swift
//  MullvadVPN
//
//  Created by pronebird on 29/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class DisconnectSplitButton: UIView {
    @IBOutlet var contentView: UIView!
    @IBOutlet var primaryButton: AppButton!
    @IBOutlet var secondaryButton: AppButton!

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

        contentView.translatesAutoresizingMaskIntoConstraints = false
        addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.leadingAnchor.constraint(equalTo: leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: trailingAnchor),
            contentView.topAnchor.constraint(equalTo: topAnchor),
            contentView.bottomAnchor.constraint(equalTo: bottomAnchor)
        ])
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        adjustTitleLabelPosition()
    }

    private func adjustTitleLabelPosition() {
        let offset = secondaryButton.frame.width + AppButton.defaultContentInsets.left

        primaryButton.contentEdgeInsets.left = offset
    }
}
