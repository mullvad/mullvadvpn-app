import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';

export const LocationSelectorLine = styled.div<{ $visible?: boolean }>`
  ${({ $visible }) => css`
    position: absolute;
    left: 24px;
    top: 0;
    height: 100%;
    width: 2px;
    background-color: ${colors.whiteAlpha60};
    z-index: var(--line-z-index);
    opacity: ${$visible ? 1 : 0};
    transition: opacity 0.15s ease-in-out;
  `}
`;
