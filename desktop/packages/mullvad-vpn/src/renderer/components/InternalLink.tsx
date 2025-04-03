import { useCallback } from 'react';

import { Link, LinkProps } from '../lib/components';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';

export type InternalLinkProps = Omit<LinkProps<'a'>, 'href' | 'as'> & {
  to: RoutePath;
};

export function InternalLink({ to, onClick, ...props }: InternalLinkProps) {
  const history = useHistory();
  const navigate = useCallback(
    (e: React.MouseEvent<HTMLAnchorElement>) => {
      e.preventDefault();
      if (onClick) {
        onClick(e);
      }
      return history.push(to);
    },
    [history, to, onClick]
  );
  return <Link href="" onClick={navigate} {...props} />;
}
