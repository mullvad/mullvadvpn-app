import { Flex } from '../../../../../lib/components';
import { ApplicationRowContextProvider } from './ApplicationRowContext';
import { StyledContainer } from './ApplicationRowStyles';
import { type ApplicationRowProps } from './ApplicationRowTypes';
import { AddButton, DeleteButton, Icon, Label, RemoveButton } from './components';
import { useShowAddButton, useShowDeleteButton, useShowRemoveButton } from './hooks';

export function ApplicationRowInner() {
  const showAddButton = useShowAddButton();
  const showDeleteButton = useShowDeleteButton();
  const showRemoveButton = useShowRemoveButton();

  return (
    <>
      <StyledContainer>
        <Icon />
        <Label />
        <Flex $gap="small">
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
