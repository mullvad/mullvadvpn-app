import React from 'react';
import styled, { css } from 'styled-components';

import { Colors, colors, Typography } from '../../foundations';
import { PolymorphicProps } from '../../types';
import { LinkIcon, LinkText, StyledIcon as StyledLinkIcon, StyledLinkText } from './components';
import { useStateColors } from './hooks';
import { LinkProvider } from './LinkContext';

type LinkBaseProps = {
  variant?: Typography;
  color?: Colors;
};

export type LinkProps<T extends React.ElementType = 'a'> = PolymorphicProps<T, LinkBaseProps>;

const StyledLink = styled.a<{
  $hoverColor: Colors;
  $activeColor: Colors;
}>(({ $hoverColor, $activeColor }) => {
  const hoverColor = colors[$hoverColor];
  const activeColor = colors[$activeColor];
  return css`
    cursor: default;
    text-decoration: none;
    display: inline;
    width: fit-content;

    &&:hover > ${StyledLinkText} {
      color: ${hoverColor};
    }

    &&:active > ${StyledLinkText} {
      color: ${activeColor};
    }

    &&:focus-visible > ${StyledLinkText} {
      outline: 2px solid ${colors.white};
      outline-offset: 2px;
    }

    &&:disabled > ${StyledLinkText} {
      color: ${colors.whiteAlpha40};
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
  color = 'chalk',
  variant,
  children,
  ...props
}: LinkProps<T>) {
  const { hover, active } = useStateColors(color);
  return (
    <LinkProvider variant={variant} color={color}>
      <StyledLink $hoverColor={hover} $activeColor={active} draggable={false} {...props}>
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
