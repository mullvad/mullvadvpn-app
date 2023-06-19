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
        view.spacing = UIMetrics.CustomAlert.containerSpacing
        view.isLayoutMarginsRelativeArrangement = true
        view.directionalLayoutMargins = UIMetrics.CustomAlert.containerMargins

        return view
    }()

    private var handlers = [UIButton: Handler]()

    init(title: String? = nil, message: String? = nil, icon: Icon? = nil) {
        super.init(nibName: nil, bundle: nil)

        modalPresentationStyle = .overFullScreen
        modalTransitionStyle = .crossDissolve

        icon.flatMap { addIcon($0) }
        title.flatMap { addTitle($0) }
        message.flatMap { addMessage($0) }

        containerView.arrangedSubviews.last.flatMap {
            containerView.setCustomSpacing(UIMetrics.CustomAlert.containerMargins.top, after: $0)
        }

        // Icon only alerts should have equal top and bottom margin.
        if title == nil, message == nil {
            containerView.directionalLayoutMargins.bottom = UIMetrics.CustomAlert.containerMargins.top
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

    func addAction(title: String, style: ActionStyle, handler: (() -> Void)? = nil) {
        // The presence of a button should reset any custom button margin to default.
        containerView.directionalLayoutMargins.bottom = UIMetrics.CustomAlert.containerMargins.bottom

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

        let style = NSMutableParagraphStyle()
        style.paragraphSpacing = 16

        label.attributedText = NSAttributedString(
            markdownString: message,
            options: .init(font: .preferredFont(forTextStyle: .body), paragraphStyle: style)
        )
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
        dismiss(animated: true) { [self] in
            if let handler = handlers[button] {
                handler()
            }
            didDismiss?()
            didDismiss = nil
            handlers.removeAll()
        }
    }
}
