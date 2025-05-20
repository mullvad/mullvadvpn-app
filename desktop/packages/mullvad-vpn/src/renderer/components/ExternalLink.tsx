import { useCallback } from 'react';
import styled from 'styled-components';

import { Url } from '../../shared/constants';
import { useAppContext } from '../context';
import { Link, LinkProps } from '../lib/components';

export type ExternalLinkProps = Omit<LinkProps<'a'>, 'href' | 'as'> & {
  to: Url;
  withAuth?: boolean;
};

const StyledLink = styled(Link)`
  display: inline-flex;
  width: fit-content;
`;

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
  return <StyledLink href="" onClick={navigate} {...props} />;
}

const ExternalLinkNamespace = Object.assign(ExternalLink, {
  Icon: Link.Icon,
});

export { ExternalLinkNamespace as ExternalLink };
