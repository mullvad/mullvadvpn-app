import { useCallback } from 'react';

import { RoutePath } from '../../shared/routes';
import { Link, LinkProps } from '../lib/components/link';
import { useHistory } from '../lib/history';

export type InternalLinkProps = Omit<LinkProps, 'href' | 'as'> & {
  to: RoutePath;
};

function InternalLink({ to, onClick, ...props }: InternalLinkProps) {
  const history = useHistory();
  const navigate = useCallback(
    (e: React.MouseEvent<HTMLAnchorElement>) => {
      e.preventDefault();
      if (onClick) {
        onClick(e);
      }
      return history.push(to);
    },
    [history, to, onClick],
  );
  return <Link href="" onClick={navigate} {...props} />;
}

const InternalLinkNamespace = Object.assign(InternalLink, {
  Text: Link.Text,
  Icon: Link.Icon,
});

export { InternalLinkNamespace as InternalLink };
