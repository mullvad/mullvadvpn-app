import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { FlexProps } from '../../../flex';

export type ListItemFooterProps = FlexProps;

export const StyledListItemFooter = styled.div`
  grid-column: 1 / -1;
  padding: 0 ${spacings.medium};
  margin-top: ${spacings.tiny};
`;

export const ListItemFooter = (props: ListItemFooterProps) => {
  return <StyledListItemFooter {...props} />;
};
