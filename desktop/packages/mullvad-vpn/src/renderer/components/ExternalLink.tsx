import { useCallback } from 'react';

import { Url } from '../../shared/constants';
import { useAppContext } from '../context';
import { Link, LinkProps } from '../lib/components';

export type ExternalLinkPops = Omit<LinkProps<'a'>, 'href' | 'as'> & {
  to: Url;
};

export const ExternalLink = ({ to, onClick, ...props }: ExternalLinkPops) => {
  const { openUrl } = useAppContext();
  const navigate = useCallback(
    (e: React.MouseEvent<HTMLAnchorElement>) => {
      e.preventDefault();
      if (onClick) {
        onClick(e);
      }
      return openUrl(to);
    },
    [onClick, openUrl, to],
  );
  return <Link href="" onClick={navigate} {...props} />;
};
