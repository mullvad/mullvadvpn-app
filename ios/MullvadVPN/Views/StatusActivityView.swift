// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import UIKit

class StatusActivityView: UIView {
    enum State {
        case hidden, activity, success, failure
    }

    var state: State = .hidden {
        didSet {
            updateView()
        }
    }

    private let statusImageView: StatusImageView = {
        let imageView = StatusImageView(style: .failure)
        imageView.translatesAutoresizingMaskIntoConstraints = false
        imageView.contentMode = .scaleAspectFit
        return imageView
    }()

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let view = SpinnerActivityIndicatorView(style: .large)
        view.tintColor = .white
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    init(state: State) {
        super.init(frame: .zero)

        self.state = state
        addSubview(statusImageView)
        addSubview(activityIndicator)

        NSLayoutConstraint.activate([
            activityIndicator.widthAnchor.constraint(equalTo: activityIndicator.heightAnchor),
            activityIndicator.heightAnchor.constraint(equalTo: statusImageView.heightAnchor),
            statusImageView.topAnchor.constraint(equalTo: topAnchor),
            statusImageView.bottomAnchor.constraint(equalTo: bottomAnchor),

            statusImageView.centerXAnchor.constraint(equalTo: centerXAnchor),
            statusImageView.centerYAnchor.constraint(equalTo: centerYAnchor),
            activityIndicator.centerXAnchor.constraint(equalTo: centerXAnchor),
            activityIndicator.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])

        updateView()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }

    private func updateView() {
        switch state {
        case .hidden:
            statusImageView.alpha = 0
            activityIndicator.stopAnimating()
        case .activity:
            statusImageView.alpha = 0
            activityIndicator.startAnimating()
        case .success:
            statusImageView.alpha = 1
            statusImageView.style = .success
            activityIndicator.stopAnimating()
        case .failure:
            statusImageView.alpha = 1
            statusImageView.style = .failure
            activityIndicator.stopAnimating()
        }
    }
}
