import styled from 'styled-components';

import { ISplitTunnelingApplication } from '../../../../../../shared/application-types';
import { Flex } from '../../../../../lib/components';
import { colors } from '../../../../../lib/foundations';
import { Container } from '../../../../cell';
import { ApplicationIcon } from '../application-icon';
import { ApplicationLabel } from '../application-label';
import { ApplicationRowContextProvider } from './ApplicationRowContext';
import { AddButton, DeleteButton, RemoveButton } from './components';
import {
  useApplication,
  useShowAddButton,
  useShowDeleteButton,
  useShowRemoveButton,
} from './hooks';

export type ApplicationRowProps = {
  application: ISplitTunnelingApplication;
  onAdd?: (application: ISplitTunnelingApplication) => void;
  onDelete?: (application: ISplitTunnelingApplication) => void;
  onRemove?: (application: ISplitTunnelingApplication) => void;
};

export const StyledContainer = styled(Container)({
  backgroundColor: colors.blue40,
});

export function ApplicationRowInner() {
  const application = useApplication();
  const showAddButton = useShowAddButton();
  const showDeleteButton = useShowDeleteButton();
  const showRemoveButton = useShowRemoveButton();

  return (
    <>
      <StyledContainer>
        <ApplicationIcon icon={application.icon} />
        <ApplicationLabel>{application.name}</ApplicationLabel>
        <Flex gap="small">
          {showAddButton && <AddButton />}
          {showDeleteButton && <DeleteButton />}
          {showRemoveButton && <RemoveButton />}
        </Flex>
      </StyledContainer>
    </>
  );
}

export function ApplicationRow(props: ApplicationRowProps) {
  return (
    <ApplicationRowContextProvider {...props}>
      <ApplicationRowInner />
    </ApplicationRowContextProvider>
  );
}
