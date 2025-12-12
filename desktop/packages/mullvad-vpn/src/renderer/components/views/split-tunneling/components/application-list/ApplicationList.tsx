import styled from 'styled-components';

import { IApplication } from '../../../../../../shared/application-types';
import { Flex, Spinner } from '../../../../../lib/components';
import { colors, spacings } from '../../../../../lib/foundations';
import { CellButton } from '../../../../cell';
import { measurements } from '../../../../common-styles';
import List from '../../../../List';
import { applicationGetKey } from './utils';

export const StyledSpinnerRow = styled(CellButton)({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: `${spacings.small} 0`,
  marginBottom: measurements.rowVerticalMargin,
  background: colors.blue40,
});

export type ApplicationListProps<T extends IApplication> = {
  applications: T[] | undefined;
  rowRenderer: (application: T) => React.ReactElement;
  'data-testid'?: string;
};

export function ApplicationList<T extends IApplication>({
  applications,
  rowRenderer,
  ...props
}: ApplicationListProps<T>) {
  if (applications == undefined) {
    return (
      <StyledSpinnerRow>
        <Spinner size="big" />
      </StyledSpinnerRow>
    );
  } else {
    const items = applications.slice().sort((a, b) => a.name.localeCompare(b.name));

    return (
      <Flex flexDirection="column" data-testid={props['data-testid']}>
        <List items={items} getKey={applicationGetKey}>
          {rowRenderer}
        </List>
      </Flex>
    );
  }
}
