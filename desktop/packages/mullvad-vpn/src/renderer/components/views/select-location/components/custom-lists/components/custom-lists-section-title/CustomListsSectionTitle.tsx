import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import {
  SectionTitle,
  type SectionTitleProps,
} from '../../../../../../../lib/components/section-title';
import { useCustomListsContext } from '../../CustomListsContext';

export type CustomListsSectionTitleProps = SectionTitleProps;

const StyledIconButton = styled(SectionTitle.IconButton)<{ $active: boolean }>`
  transform: ${({ $active }) => (!$active ? 'rotate(45deg)' : 'rotate(0deg)')};
  transition: transform 0.2s ease-in-out;
`;

export function CustomListsSectionTitle({ ...props }: CustomListsSectionTitleProps) {
  const { addFormVisible, hideAddForm, showAddForm } = useCustomListsContext();
  const handleOnClick = addFormVisible ? hideAddForm : showAddForm;
  const createAriaLabel = messages.pgettext('accessibility', 'Create new custom list');
  const cancelAriaLabel = messages.pgettext('accessibility', 'Cancel creating new custom list');

  return (
    <SectionTitle {...props}>
      <SectionTitle.Title>
        {messages.pgettext('select-location-view', 'Custom lists')}
      </SectionTitle.Title>
      <SectionTitle.Divider />
      <StyledIconButton
        $active={addFormVisible}
        onClick={handleOnClick}
        aria-label={addFormVisible ? cancelAriaLabel : createAriaLabel}>
        <SectionTitle.IconButton.Icon icon="cross" />
      </StyledIconButton>
    </SectionTitle>
  );
}
