import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { FlexProps } from '../../../flex';
import { ListItemFooterText } from './components';

export type ListItemFooterProps = FlexProps;

export const StyledListItemFooter = styled.div`
  grid-column: 1 / -1;
  padding: 0 ${spacings.medium};
  margin-top: ${spacings.tiny};
`;

const ListItemFooter = (props: ListItemFooterProps) => {
  return <StyledListItemFooter {...props} />;
};

const ListItemFooterNamespace = Object.assign(ListItemFooter, {
  Text: ListItemFooterText,
});

export { ListItemFooterNamespace as ListItemFooter };
