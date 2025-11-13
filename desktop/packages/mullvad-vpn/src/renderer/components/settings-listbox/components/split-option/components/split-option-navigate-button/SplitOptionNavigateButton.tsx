import { useCallback } from 'react';
import styled from 'styled-components';

import { RoutePath } from '../../../../../../../shared/routes';
import { Flex, Icon } from '../../../../../../lib/components';
import { colors } from '../../../../../../lib/foundations';
import { useHistory } from '../../../../../../lib/history';

export type NavigationOptionNavigateProps = {
  to: RoutePath;
} & React.ComponentPropsWithRef<'button'>;

const StyledFlex = styled(Flex)`
  background-color: ${colors.blue60};
  height: 100%;
`;

const StyledSplitOptionNavigateButton = styled.button`
  position: relative;
  margin-bottom: 1px;
  &&::before {
    content: '';
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 1px;
    height: 22px;
    background-color: ${colors.darkBlue};
  }
  &&:hover {
    ${StyledFlex} {
      background-color: ${colors.blue};
    }
  }
  &&:active {
    ${StyledFlex} {
      background-color: ${colors.whiteOnBlue20};
    }
  }
  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -2px;
    z-index: 10;
  }
`;

export function SplitOptionNavigateButton({
  to,
  children,
  ...props
}: NavigationOptionNavigateProps) {
  const history = useHistory();
  const navigate = useCallback(() => {
    return history.push(to);
  }, [history, to]);

  return (
    <StyledSplitOptionNavigateButton data-split-button onClick={navigate} tabIndex={-1} {...props}>
      <StyledFlex justifyContent="center" alignItems="center" padding={{ horizontal: 'medium' }}>
        <Icon icon={'chevron-right'} aria-hidden="true" />
      </StyledFlex>
    </StyledSplitOptionNavigateButton>
  );
}
