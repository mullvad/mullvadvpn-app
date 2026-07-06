import styled from 'styled-components';

import { messages } from '../../../../../shared/gettext';
import { Flex } from '../../../../lib/components/flex';
import CustomScrollbars from '../../../CustomScrollbars';
import { StatusDialog, type StatusDialogProps } from '../../../status-dialog';
import { useInfoContext } from '../../InfoContext';

export type InfoDialogProps = Omit<StatusDialogProps, 'variant'> & {
  buttons?: React.ReactNode[];
};

const StyledFlex = styled(Flex)`
  overflow: hidden;
`;

const StyledCustomScrollbars = styled(CustomScrollbars)({
  paddingRight: '16px',
});

function InfoDialog({ children, buttons, ...props }: InfoDialogProps) {
  const { open, onOpenChange } = useInfoContext();

  return (
    <StatusDialog variant="info" open={open} onOpenChange={onOpenChange} {...props}>
      <StyledFlex flexDirection="column" gap="small">
        <StyledCustomScrollbars>{children}</StyledCustomScrollbars>
        {buttons}
        <StatusDialog.CloseButton>
          <StatusDialog.CloseButton.Text>
            {messages.gettext('Got it!')}
          </StatusDialog.CloseButton.Text>
        </StatusDialog.CloseButton>
      </StyledFlex>
    </StatusDialog>
  );
}

const InfoDialogNamespace = Object.assign(InfoDialog, {
  Text: StatusDialog.Text,
  Title: StatusDialog.Title,
  Button: StatusDialog.Button,
});

export { InfoDialogNamespace as InfoDialog };
