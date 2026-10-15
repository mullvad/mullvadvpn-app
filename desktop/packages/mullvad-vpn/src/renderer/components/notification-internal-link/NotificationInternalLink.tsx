import { TransitionType } from '../../lib/history';
import { createComponentNamespace } from '../../utils';
import { InternalLink, type InternalLinkProps } from '../internal-link';

export type NotificationInternalLinkProps = InternalLinkProps;

function NotificationInternalLink({
  locationState: locationStateProps,
  ...props
}: NotificationInternalLinkProps) {
  const locationState = { transition: TransitionType.show, ...locationStateProps };
  return <InternalLink variant="labelTinySemiBold" locationState={locationState} {...props} />;
}

const NotificationInternalLinkNamespace = createComponentNamespace(NotificationInternalLink, {
  Text: InternalLink.Text,
  Icon: InternalLink.Icon,
});

export { NotificationInternalLinkNamespace as NotificationInternalLink };
