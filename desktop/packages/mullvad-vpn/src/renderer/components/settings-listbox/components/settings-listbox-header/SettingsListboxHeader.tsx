import styled from 'styled-components';

import { useScrollToListItem } from '../../../../hooks';
import { Listbox } from '../../../../lib/components/listbox';
import { ListboxHeaderProps } from '../../../../lib/components/listbox/components';
import { useSettingsListboxContext } from '../../SettingsListboxContext';

export type SettingsListboxHeaderProps = ListboxHeaderProps;

export const StyledSettingsListboxHeader = styled(Listbox.Header)`
  margin-bottom: 1px;
`;

export function SettingsListboxHeader({ children, ...props }: SettingsListboxHeaderProps) {
  const { anchorId } = useSettingsListboxContext();
  const { ref, animation } = useScrollToListItem(anchorId);
  return (
    <StyledSettingsListboxHeader ref={ref} animation={animation} {...props}>
      {children}
    </StyledSettingsListboxHeader>
  );
}
