import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { FlexProps } from '../../../flex';
import { StyledIconButtonIcon } from '../../../icon-button/components';
import { StyledText } from '../../../text';
import { StyledTextField } from '../../../text-field';

export type ListItemActionProps = FlexProps;

export const StyledListItemActionGroup = styled.div`
  display: grid;
  grid-template-columns: 48px;

  // Center children in columns
  > :is(:nth-child(1), :nth-child(2)) {
    justify-self: center;
    align-self: center;
  }
  // If there is two children, set up with fixed widths
  &&:has(> :last-child:nth-child(2)) {
    grid-template-columns: auto 48px;
  }

  // If there is two children, with icon button as first child, set up with fixed widths
  &&:has(> :first-child ${StyledIconButtonIcon}):has(> :last-child:nth-child(2)) {
    grid-template-columns: 32px 48px;
  }

  // If there is a single text field child, make it take automatic width with margin
  &&:has(${StyledTextField}):has(> :last-child:nth-child(1)) {
    grid-template-columns: auto;
    margin-right: ${spacings.small};
  }

  // If there is a single text child, make it take automatic width with margin
  &&:has(${StyledText}):has(> :last-child:nth-child(1)) {
    grid-template-columns: auto;
    margin-right: ${spacings.medium};
  }
`;

export const ListItemActionGroup = (props: ListItemActionProps) => {
  return <StyledListItemActionGroup {...props} />;
};
