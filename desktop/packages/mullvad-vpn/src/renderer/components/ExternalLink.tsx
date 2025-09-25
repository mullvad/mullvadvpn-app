import { useCallback } from 'react';

import { Url } from '../../shared/constants';
import { useAppContext } from '../context';
import { Link, LinkProps } from '../lib/components/link';

export type ExternalLinkProps = Omit<LinkProps, 'href' | 'as'> & {
  to: Url;
  withAuth?: boolean;
};

function ExternalLink({ to, onClick, withAuth, ...props }: ExternalLinkProps) {
  const { openUrl, openUrlWithAuth } = useAppContext();
  const navigate = useCallback(
    (e: React.MouseEvent<HTMLAnchorElement>) => {
      e.preventDefault();
      if (onClick) {
        onClick(e);
      }

      if (withAuth) {
        return openUrlWithAuth(to);
      }
      return openUrl(to);
    },
    [onClick, openUrl, openUrlWithAuth, to, withAuth],
  );
  return <Link href="" onClick={navigate} {...props} />;
}

const ExternalLinkNamespace = Object.assign(ExternalLink, {
  Text: Link.Text,
  Icon: Link.Icon,
});

export { ExternalLinkNamespace as ExternalLink };
