import React from 'react';
import styled, { css } from 'styled-components';

import { Colors, colors, Radius, Typography } from '../../foundations';
import { PolymorphicProps, TransientProps } from '../../types';
import { LinkIcon, LinkText, StyledIcon as StyledLinkIcon, StyledLinkText } from './components';
import { useHoverColor } from './hooks';
import { LinkProvider } from './LinkContext';

type LinkBaseProps = {
  variant?: Typography;
  color?: Colors;
};

export type LinkProps<T extends React.ElementType = 'a'> = PolymorphicProps<T, LinkBaseProps>;

const StyledLink = styled.a<
  TransientProps<LinkProps> & {
    $hoverColor?: Colors;
  }
>(({ $hoverColor }) => {
  return css`
    cursor: default;
    text-decoration: none;
    display: inline;
    width: fit-content;

    &&:hover > ${StyledLinkText} {
      text-decoration-line: underline;
      text-underline-offset: 2px;
      color: ${$hoverColor};
    }

    &&:focus-visible > ${StyledLinkText} {
      border-radius: ${Radius.radius4};
      outline: 2px solid ${colors.white};
      outline-offset: 2px;
    }

    > ${StyledLinkIcon}:first-child:not(:only-child) {
      margin-right: 2px;
    }
    > ${StyledLinkIcon}:last-child:not(:only-child) {
      margin-left: 2px;
    }
  `;
});

function Link<T extends React.ElementType = 'a'>({
  color,
  variant,
  children,
  ...props
}: LinkProps<T>) {
  const hoverColor = useHoverColor(color);
  return (
    <LinkProvider variant={variant} color={color}>
      <StyledLink $hoverColor={hoverColor} {...props}>
        {children}
      </StyledLink>
    </LinkProvider>
  );
}

const LinkNamespace = Object.assign(Link, {
  Text: LinkText,
  Icon: LinkIcon,
});

export { LinkNamespace as Link };
