//
//  CustomAlertController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-19.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomAlertViewController: UIViewController {
    typealias Handler = () -> Void

    enum Icon {
        case alert
        case info
        case spinner

        var image: UIImage? {
            switch self {
            case .alert:
                return UIImage(named: "IconAlert")?.withTintColor(.dangerColor)
            case .info:
                return UIImage(named: "IconInfo")?.withTintColor(.white)
            default:
                return nil
            }
        }
    }

    enum ActionStyle {
        case `default`
        case destructive

        fileprivate var buttonStyle: AppButton.Style {
            switch self {
            case .default:
                return .default
            case .destructive:
                return .danger
            }
        }
    }

    var didDismiss: (() -> Void)?

    private let containerView: UIStackView = {
        let view = UIStackView()

        view.axis = .vertical
        view.backgroundColor = .secondaryColor
        view.layer.cornerRadius = 11
        view.spacing = UIMetrics.customAlertContainerSpacing
        view.isLayoutMarginsRelativeArrangement = true
        view.directionalLayoutMargins = UIMetrics.customAlertContainerMargins

        return view
    }()

    private var handlers = [UIButton: Handler]()

    convenience init(title: String? = nil, message: String?, icon: Icon? = nil) {
        self.init(title: title, messages: message.flatMap { [$0] } ?? [], icon: icon)
    }

    init(title: String? = nil, messages: [String] = [], icon: Icon? = nil) {
        super.init(nibName: nil, bundle: nil)

        modalPresentationStyle = .overFullScreen
        modalTransitionStyle = .crossDissolve

        icon.flatMap { addIcon($0) }
        title.flatMap { addTitle($0) }
        messages.forEach { addMessage($0) }

        containerView.arrangedSubviews.last.flatMap {
            containerView.setCustomSpacing(UIMetrics.customAlertContainerMargins.top, after: $0)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .black.withAlphaComponent(0.5)

        view.addConstrainedSubviews([containerView]) {
            containerView.pinEdges(.init([.leading(0), .trailing(0)]), to: view.layoutMarginsGuide)
            containerView.centerYAnchor.constraint(equalTo: view.centerYAnchor)
        }
    }

    override func viewDidDisappear(_ animated: Bool) {
        super.viewDidDisappear(animated)
        handlers.removeAll()
    }

    func addAction(title: String, style: ActionStyle, handler: (() -> Void)? = nil) {
        let button = AppButton(style: style.buttonStyle)

        button.addTarget(self, action: #selector(didTapButton), for: .touchUpInside)
        button.setTitle(title, for: .normal)

        containerView.addArrangedSubview(button)
        handler.flatMap { handlers[button] = $0 }
    }

    private func addTitle(_ title: String) {
        let label = UILabel()

        label.text = title
        label.font = .preferredFont(forTextStyle: .title3, weight: .semibold)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        containerView.addArrangedSubview(label)
    }

    private func addMessage(_ message: String) {
        let label = UILabel()

        label.text = message
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .white.withAlphaComponent(0.8)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        containerView.addArrangedSubview(label)
    }

    private func addIcon(_ icon: Icon) {
        let iconView = icon == .spinner ? getSpinnerView() : getImageView(for: icon)
        containerView.addArrangedSubview(iconView)
    }

    private func getImageView(for icon: Icon) -> UIView {
        let imageView = UIImageView()
        let imageContainerView = UIView()

        imageContainerView.addConstrainedSubviews([imageView]) {
            imageView.pinEdges(.init([.top(0), .bottom(0)]), to: imageContainerView)
            imageView.centerXAnchor.constraint(equalTo: imageContainerView.centerXAnchor, constant: 0)
            imageView.heightAnchor.constraint(equalToConstant: 44)
            imageView.widthAnchor.constraint(equalTo: imageView.heightAnchor, multiplier: 1)
        }

        imageView.image = icon.image?.withRenderingMode(.alwaysOriginal)
        imageView.contentMode = .scaleAspectFit

        return imageContainerView
    }

    private func getSpinnerView() -> UIView {
        let spinnerView = SpinnerActivityIndicatorView(style: .large)
        let spinnerContainerView = UIView()

        spinnerContainerView.addConstrainedSubviews([spinnerView]) {
            spinnerView.pinEdges(.init([.top(0), .bottom(0)]), to: spinnerContainerView)
            spinnerView.centerXAnchor.constraint(equalTo: spinnerContainerView.centerXAnchor, constant: 0)
        }

        spinnerView.startAnimating()

        return spinnerContainerView
    }

    @objc private func didTapButton(_ button: AppButton) {
        dismiss(animated: true, completion: didDismiss)

        if let handler = handlers[button] {
            handler()
        }
    }
}
