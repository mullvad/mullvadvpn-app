import { useCallback } from 'react';
import styled from 'styled-components';

import { Url } from '../../shared/constants';
import { useAppContext } from '../context';
import { Link, LinkProps } from '../lib/components';

export type ExternalLinkProps = Omit<LinkProps<'a'>, 'href' | 'as'> & {
  to: Url;
};

const StyledLink = styled(Link)`
  display: inline-flex;
  width: fit-content;
`;

function ExternalLink({ to, onClick, ...props }: ExternalLinkProps) {
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
  return <StyledLink href="" onClick={navigate} {...props} />;
}

const ExternalLinkNamespace = Object.assign(ExternalLink, {
  Icon: Link.Icon,
});

export { ExternalLinkNamespace as ExternalLink };
