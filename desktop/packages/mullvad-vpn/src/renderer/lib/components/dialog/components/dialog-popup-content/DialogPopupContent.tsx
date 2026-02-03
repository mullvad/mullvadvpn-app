import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { Alert, type AlertProps } from '../../../alert';
import { StyledAlertIcon } from '../../../alert/components';

export type DialogPopupContentProps = AlertProps;

const StyledDialogContainer = styled(Alert)`
  > ${StyledAlertIcon}:first-child {
    margin-top: ${spacings.small};
  }
`;
export function DialogPopupContent(props: DialogPopupContentProps) {
  return <StyledDialogContainer {...props} />;
}
