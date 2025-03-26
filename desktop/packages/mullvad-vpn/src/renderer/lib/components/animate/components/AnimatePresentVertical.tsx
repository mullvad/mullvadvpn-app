import React from 'react';
import styled from 'styled-components';

export interface AnimatePresentVerticalProps extends React.HTMLAttributes<HTMLDivElement> {
  present?: boolean;
  children?: React.ReactNode;
}

const StyledDiv = styled.div`
  --display-start: none;
  --height-start: 0;
  --display-end: block;
  --height-end: min-content;

  overflow: clip;
  transition-property: display, height;
  transition-duration: 0.25s;
  transition-timing-function: ease;
  interpolate-size: allow-keywords;
  transition-behavior: allow-discrete;
  display: var(--display-start);
  height: var(--height-start);
  &&[data-present='true'] {
    display: var(--display-end);
    height: var(--height-end);
    @starting-style {
      height: var(--height-start);
    }
  }
`;

export function AnimatePresentVertical({ present, ...props }: AnimatePresentVerticalProps) {
  return <StyledDiv data-present={present} {...props} />;
}
