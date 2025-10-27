import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';

type PageIndicatorProps = React.ComponentPropsWithRef<'button'> & {
  pageNumber: number;
  goToPage: (page: number) => void;
};

const StyledPageIndicator = styled.button`
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: ${colors.whiteAlpha80};
  &&:hover {
    background-color: ${colors.whiteAlpha80};
  }
  &&:disabled {
    background-color: ${colors.whiteAlpha40};
  }
  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: '2px';
  }
`;

export function PageIndicator(props: PageIndicatorProps) {
  const { goToPage } = props;

  const onClick = React.useCallback(() => {
    goToPage(props.pageNumber);
  }, [goToPage, props.pageNumber]);

  return <StyledPageIndicator onClick={onClick} {...props} />;
}
